#!/usr/bin/env python3

import os
import shutil
import string
import sys
import fileinput
import re

import muncher
from muncher import MuncherError

## refactor transpiled sqlite3 code to make it more readable 


def replace_text(buffer: str, search: str, replace:str=''):

    newbuffer = buffer.replace(search, replace)
    if newbuffer != buffer:
        if(replace == ''):
            print("    remove text '{}'".format(search))
        else:
            print("    replace text '{}' --> '{}'".format(search, replace))

    return newbuffer

def remove_type(buffer: str, types: list):
    for ty in types:
        newbuffer = re.sub(r'pub type {} = .*;'.format(ty), '', buffer)
        if newbuffer != buffer:
            print("    remove type '{}'".format(ty))
            buffer = newbuffer

    return buffer    

## replace primitive type using native rust
def replace_primitive(filename: str):
    print("{}: replace primitive type to using native rust version".format(filename))
    
    buffer = read(filename)
    
    buffer = remove_type(buffer, ['u8_0', 'i8_0', 'u16_0', 'i16_0', 'u32_0', 'u64_0', 'i64_0'])

    buffer = replace_text(buffer, 'u8_0', 'u8')
    buffer = replace_text(buffer, 'i8_0', 'i8')
    buffer = replace_text(buffer, 'u16_0', 'u16')
    buffer = replace_text(buffer, 'i16_0', 'i16')
    buffer = replace_text(buffer, 'u32_0', 'u32')
    buffer = replace_text(buffer, 'u64_0', 'u64')
    buffer = replace_text(buffer, 'i64_0', 'i64')

    buffer = remove_type(buffer, ['sqlite3_int64', 'sqlite3_uint64', 'sqlite_int64', 'sqlite_uint64'])

    buffer = replace_text(buffer, 'sqlite_int64', 'i64')
    buffer = replace_text(buffer, 'sqlite_uint64', 'u64')

    buffer = replace_text(buffer, 'sqlite3_int64', 'i64')
    buffer = replace_text(buffer, 'sqlite3_uint64', 'u64')

    buffer = remove_type(buffer, ['i64', 'u64'])

    ## libc is last 
    buffer = replace_text(buffer, 'libc::c_ulonglong', 'u64')
    buffer = replace_text(buffer, 'libc::c_longlong', 'i64')
    buffer = replace_text(buffer, 'libc::c_short', 'i16')
    buffer = replace_text(buffer, 'libc::c_ushort', 'u16')
    buffer = replace_text(buffer, 'libc::c_int', 'i32')
    buffer = replace_text(buffer, 'libc::c_uint', 'u32')
    buffer = replace_text(buffer, 'libc::c_ulong', 'u64')
    buffer = replace_text(buffer, 'libc::c_uchar', 'u8')
    buffer = replace_text(buffer, 'libc::c_char', 'i8')
    buffer = replace_text(buffer, 'libc::c_float', 'f32')
    buffer = replace_text(buffer, 'libc::c_double', 'f64')
    
    commit(filename, buffer)

## remove useless cast
def remove_redundant_cast(filename):
    print("{}: replace redundant cast".format(filename))
    buffer = read(filename)
    buffer = replace_text(buffer, ' 0 as i32', ' 0')
    buffer = replace_text(buffer, ' 0 as i64', ' 0')
    buffer = replace_text(buffer, ' 0 as u64', ' 0')
    buffer = replace_text(buffer, ' 0 as u8', ' 0')
    buffer = replace_text(buffer, ' 0 as i8', ' 0')
    #buffer = replace_text(buffer, '-(1 as i32) as', '-1 as') ## causing compiler error

    buffer = re.sub(r'(\d) as i32 as i8', r'\1 as i8', buffer)
    buffer = re.sub(r'(\d) as i32 as u8', r'\1 as u8', buffer)
    buffer = re.sub(r'(\d) as i32 as isize', r'\1 as isize', buffer)
    buffer = re.sub(r'(\d) as i32 as i16', r'\1 as i16', buffer)
    buffer = re.sub(r'(\d) as i32 as u16', r'\1 as u16', buffer)

    buffer = re.sub(r'\[(\d+) as i32 as usize\]', r'[\1]', buffer)
    buffer = re.sub(r'\.offset\((\d+) as isize\)', r'.offset(\1)', buffer)
    

    commit(filename, buffer)

__CACHE = {}
def read(filename):
    if __CACHE.get(filename) == None:
        tempFile = open( filename, 'r' )
        buffer = tempFile.read()
        tempFile.close()
        __CACHE[filename] = buffer
    
    return __CACHE[filename]

def commit(filename, buffer):
    if __CACHE[filename] != buffer:
        print("    commited changes to files...")
        tempFile = open( filename, 'w')
        tempFile.write(buffer)
        tempFile.close()
        del __CACHE[filename]


def refactor_assert_as_unreachable(filename):
    print("{}: refactor __assert_fail --> unreachable!() expression".format(filename))
    buffer = read(filename)
    pos = 0

    while True:
        expr = '__assert_fail('
        pos = assert_pos = buffer.find(expr, pos)

        if pos == -1:
            break

        if buffer[pos-3:pos] == 'fn ':
            pos += len(expr)
            continue

        pos += len(expr) - 1
        argument_span = muncher.find_matching_pair_span(buffer, pos, '(', ')')
        assert_span = (assert_pos, argument_span[1])

        rewrite_text = "unreachable!()"
        buffer = muncher.replace_span(buffer, assert_span, rewrite_text)

    commit(filename, buffer)

def refactor_assert_expression(filename):
    print("{}: refactor __assert_fail --> assert!() expression".format(filename))
    buffer = read(filename)
    pos = 0

    while True:
        expr = '__assert_fail('
        pos = buffer.find(expr, pos)

        if pos == -1:
            break

        assert_pos = pos
        try:
            # backward find token 'if' <expr> '{' <then_expr> '}' 'else' '{'
            pos = brace_pos = muncher.backward_expect(buffer, pos, '{')
            pos = muncher.backward_expect(buffer, pos, 'else')
            pos = end_then_pos = muncher.backward_expect(buffer, pos, '}')
            pos = end_condition_pos = muncher.backward_until(buffer, pos, '{')
            pos = if_pos = muncher.backward_until(buffer, pos, 'if')

            condition_span = (pos + 3, end_condition_pos)
            else_span = muncher.find_matching_pair_span(buffer, brace_pos, '{', '}')
            if_span = (if_pos, else_span[1])
            then_span = (end_condition_pos + 1, end_then_pos)

            # rewrite it as '(assert!(<condition>); then_expr)' expression
            condition_text = buffer[condition_span[0]:condition_span[1]]
            then_text = buffer[then_span[0]:then_span[1]]
            rewrite_text = r"{{assert!({0}); {1}}}".format(condition_text, then_text)
            buffer = muncher.replace_span(buffer, if_span, rewrite_text)

            print("    rewrite {} - {}".format(if_span[0], if_span[1]))
            pos = if_span[0]

        except muncher.MuncherError:
            print("    __assert_fail() in {} don't match with pattern. SKIP".format(pos))
            pos = assert_pos + len(expr)
            continue

    commit(filename, buffer)


def refactor_assert_statement(filename):
    print("{}: refactor __assert_fail --> assert!() statement".format(filename))
    buffer = read(filename)
    pos = 0

    while True:
        expr = '__assert_fail('
        pos = buffer.find(expr, pos)

        if pos == -1:
            break

        assert_pos = pos
        try:
            # backward find token 'if' <expr> '{' '}' 'else' '{'
            pos = brace_pos = muncher.backward_expect(buffer, pos, '{')
            pos = muncher.backward_expect(buffer, pos, 'else')
            pos = muncher.backward_expect(buffer, pos, '}')
            pos = end_condition_pos = muncher.backward_expect(buffer, pos, '{')
            pos = if_pos = muncher.backward_until(buffer, pos, 'if')

            condition_span = (pos + 3, end_condition_pos)
            else_span = muncher.find_matching_pair_span(buffer, brace_pos, '{', '}')
            if_span = (if_pos, else_span[1])

            # rewrite it as 'assert!(<condition>);' statement
            condition_text = buffer[condition_span[0]:condition_span[1]]
            rewrite_text = "assert!({});".format(condition_text)
            buffer = muncher.replace_span(buffer, if_span, rewrite_text)

            print("    rewrite {} - {}".format(if_span[0], if_span[1]))
            pos = if_span[0]

        except muncher.MuncherError:
            print("    __assert_fail() in {} don't match with pattern. SKIP".format(pos))
            pos = assert_pos + len(expr)
            continue

    commit(filename, buffer)

def refactor_null_pointer(filename):
    print("{}: replace 0 -> ptr::null() / ptr::null_mut()".format(filename))

    as_mut = lambda b, text: replace_text(b, text, ' std::ptr::null_mut()')

    buffer = read(filename)

    # mut
    buffer = as_mut(buffer, ' 0 as *mut libc::c_void')
    buffer = as_mut(buffer, ' 0 as *const libc::c_void as *mut libc::c_void')
    buffer = as_mut(buffer, ' 0 as *const i8 as *mut i8')
    buffer = as_mut(buffer, ' 0 as *mut i8')
    #buffer = as_mut(buffer, ' 0 as *mut i32') ### causing compile error

    buffer = as_mut(buffer, ' 0 as *mut tm')

    buffer = as_mut(buffer, ' 0 as *mut unixInodeInfo')
    buffer = as_mut(buffer, ' 0 as *mut unixFile')
    buffer = as_mut(buffer, ' 0 as *mut unixShmNode')
    

    #buffer = as_mut(buffer, ' 0 as *mut sqlite3_backup') ### causing compile error
    #buffer = as_mut(buffer, ' 0 as *mut sqlite3_context') ### causing compile error
    buffer = as_mut(buffer, ' 0 as *mut sqlite3_file')
    buffer = as_mut(buffer, ' 0 as *const sqlite3_mutex as *mut sqlite3_mutex')
    buffer = as_mut(buffer, ' 0 as *const sqlite3_vfs as *mut sqlite3_vfs')
    buffer = as_mut(buffer, ' 0 as *mut sqlite3_mutex')
    #buffer = as_mut(buffer, ' 0 as *mut sqlite3_stmt') ### causing compile error
    buffer = as_mut(buffer, ' 0 as *mut sqlite3_pcache_page')
    #buffer = as_mut(buffer, ' 0 as *mut sqlite3_pcache') ### causing compile error
    #buffer = as_mut(buffer, ' 0 as *mut sqlite3_value') ### causing compile error
    buffer = as_mut(buffer, ' 0 as *mut sqlite3_vtab_cursor')
    buffer = as_mut(buffer, ' 0 as *mut sqlite3_vtab')
    buffer = as_mut(buffer, ' 0 as *mut sqlite3_vfs')

    #buffer = as_mut(buffer, ' 0 as *mut sqlite3')

    buffer = as_mut(buffer, ' 0 as *mut Btree')
    buffer = as_mut(buffer, ' 0 as *const FuncDef as *mut FuncDef')
    buffer = as_mut(buffer, ' 0 as *mut FuncDestructor')
    buffer = as_mut(buffer, ' 0 as *mut FuncDef')
    buffer = as_mut(buffer, ' 0 as *mut LookasideSlot')
    buffer = as_mut(buffer, ' 0 as *mut JsonNode')
    buffer = as_mut(buffer, ' 0 as *mut HashElem')
    buffer = as_mut(buffer, ' 0 as *mut TableLock')
    buffer = as_mut(buffer, ' 0 as *mut Table')
    buffer = as_mut(buffer, ' 0 as *mut TriggerPrg')
    buffer = as_mut(buffer, ' 0 as *mut TriggerStep')
    buffer = as_mut(buffer, ' 0 as *mut Trigger')
    buffer = as_mut(buffer, ' 0 as *mut RenameToken')
    buffer = as_mut(buffer, ' 0 as *mut ValueList')
    buffer = as_mut(buffer, ' 0 as *mut VdbeCursor')
    buffer = as_mut(buffer, ' 0 as *mut VdbeFrame')
    buffer = as_mut(buffer, ' 0 as *mut VdbeSorter')
    buffer = as_mut(buffer, ' 0 as *mut VdbeOp')
    buffer = as_mut(buffer, ' 0 as *mut Vdbe')
    

    # const
    def as_const(b, types):
        for ty in types:
            b = replace_text(b, f' 0 as {ty}', ' std::ptr::null()')
        return b

    buffer = as_const(buffer, [
        '*const u8', 
#        '*const i8', ### causing compile error
        '*const libc::c_void',
        '*const et_info',
        '*const sqlite3_mutex_methods',
        '*const sqlite3_io_methods',
    ])

    commit(filename, buffer)

    ## we run `cargo fmt` to reformat the source code because our refactoring is based on dumb search & replace,
    ## some case is missed because of new line in the code.
    print("running cargo fmt...")
    os.system('cargo fmt')

    print("{}: replace 0 -> ptr::null() / ptr::null_mut()".format(filename))
    buffer = as_mut(buffer, ' 0 as *const i8 as *mut i8')

def remove_unused_fn_definition(filename):
    print("{}: remove unused fn definition".format(filename))
    buffer = read(filename)

    functions = {}

    matches = re.finditer(r'fn (\w+)\(', buffer)
    for m in matches:
        start = m.start()

        # find until matching ')'
        #print(buffer[m.start():m.end()])
        param_span = muncher.find_matching_pair_span(buffer, m.end()-1, '(', ')')
        
        brace_pos = buffer.find('{', param_span[1])
        semicolon_pos = buffer.find(';', param_span[1])

        # function definition with body. SKIP
        if brace_pos != -1 and semicolon_pos > brace_pos:
            continue
        elif semicolon_pos != -1 and semicolon_pos < brace_pos:
            end = semicolon_pos
        else:
            continue

        span = (start, end+1)
        functions[m.group(1)] = span
        
    unuseds = {}
    for (name, span) in functions.items():
        ## find usage count
        pattern = "{}\(".format(name)
        matches = re.findall(pattern, buffer)
        count = len(matches)
        if count <= 1:
            unuseds[name] = span

    # remove unused
    newbuffer = []
    last_pos = 0
    for (name, span) in unuseds.items():
        print("    removing {}".format(name))

        content = buffer[last_pos: span[0]]
        newbuffer.append(content)
        last_pos=span[1]
    
    newbuffer.append(buffer[last_pos:])
    newbuffer = ''.join(newbuffer)
    commit(filename, newbuffer)


def main():
    print("copy transpiled from transpiled folder to src folder...")

    files = ['sqlite3.rs']
    for filename in files:
        shutil.copy(f"transpiled/{filename}", "src/")

    print("refactoring...")
    for filename in files:
        print("run cargo fmt")
        os.system('cargo fmt')

        filename = f"src/{filename}"

        replace_primitive(filename)
        remove_redundant_cast(filename)
        refactor_assert_statement(filename)
        refactor_assert_expression(filename)
        refactor_assert_as_unreachable(filename)
        refactor_null_pointer(filename)
        #remove_unused_fn_definition(filename)


if __name__ == '__main__':
    main()
