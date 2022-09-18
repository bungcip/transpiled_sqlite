#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
#![register_tool(c2rust)]
#![feature(io_safety)]
#![feature(c_variadic, extern_types, label_break_value, register_tool)]

/// helper macro to creating string from null terminated string
macro_rules! cstr_to_string {
    ($e: expr) => {
        std::ffi::CStr::from_ptr($e as *const i8).to_string_lossy()
    };
}
macro_rules! to_cstring {
    ($e: expr) => {
        std::ffi::CString::new($e).unwrap().as_ptr()
    };
}

use ascii_utils;
use ascii_utils::Check;
use bstr::{io::BufReadExt, ByteSlice};
use bstr::{BStr, B};
use const_format::concatcp;
use memmem;
use memmem::Searcher;
use rustyline;
use rustyline::error::ReadlineError;
use shell_words;
use smol_str::SmolStr;
use sqrite;
use sscanf;
use strfmt::FmtError;
use strfmt::{strfmt, strfmt_builder};

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt::{self, Write};
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::ops::Deref;
use std::os::unix::io::AsFd;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::{
    fs::{File, Permissions},
    io::{self, Write as IoWrite},
    os::unix::prelude::{FromRawFd, MetadataExt},
};

extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type sqlite3;
    pub type sqlite3_api_routines;
    pub type sqlite3_stmt;
    pub type sqlite3_value;
    pub type sqlite3_context;
    pub type sqlite3_str;
    pub type sqlite3_backup;
    pub type __dirstream;
    fn malloc(_: u64) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: u64) -> *mut libc::c_void;
    fn free(__ptr: *mut libc::c_void);
    fn getenv(__name: *const i8) -> *mut i8;
    fn system(__command: *const i8) -> i32;
    fn strncpy(_: *mut i8, _: *const i8, _: u64) -> *mut i8;
    fn strcmp(_: *const i8, _: *const i8) -> i32;
    fn strncmp(_: *const i8, _: *const i8, _: u64) -> i32;
    fn strdup(_: *const i8) -> *mut i8;
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: u64) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: i32, _: u64) -> *mut libc::c_void;
    fn memcmp(_: *const libc::c_void, _: *const libc::c_void, _: u64) -> i32;
    fn memmove(_: *mut libc::c_void, _: *const libc::c_void, _: u64) -> *mut libc::c_void;
    fn strstr(_: *const i8, _: *const i8) -> *mut i8;
    fn strlen(_: *const i8) -> u64;
    static mut stdin: *mut FILE;
    static mut stdout: *mut FILE;
    static mut stderr: *mut FILE;
    fn fclose(__stream: *mut FILE) -> i32;
    fn fflush(__stream: *mut FILE) -> i32;
    fn fopen(_: *const i8, _: *const i8) -> *mut FILE;
    fn setvbuf(__stream: *mut FILE, __buf: *mut i8, __modes: i32, __n: size_t) -> i32;
    fn fprintf(_: *mut FILE, _: *const i8, _: ...) -> i32;
    fn fgetc(__stream: *mut FILE) -> i32;
    fn fputc(__c: i32, __stream: *mut FILE) -> i32;
    fn putc(__c: i32, __stream: *mut FILE) -> i32;
    fn fgets(__s: *mut i8, __n: i32, __stream: *mut FILE) -> *mut i8;
    fn fread(_: *mut libc::c_void, _: u64, _: u64, _: *mut FILE) -> u64;
    fn fwrite(_: *const libc::c_void, _: u64, _: u64, _: *mut FILE) -> u64;
    fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: i32) -> i32;
    fn ftell(__stream: *mut FILE) -> libc::c_long;
    fn rewind(__stream: *mut FILE);
    fn popen(__command: *const i8, __modes: *const i8) -> *mut FILE;
    fn pclose(__stream: *mut FILE) -> i32;
    fn sqlite3_libversion() -> *const i8;
    fn sqlite3_sourceid() -> *const i8;
    fn sqlite3_libversion_number() -> i32;
    fn sqlite3_close(_: *mut sqlite3) -> i32;
    fn sqlite3_exec(
        _: *mut sqlite3,
        sql: *const i8,
        callback_0: Option<unsafe extern "C" fn(*mut libc::c_void, i32, *mut *mut i8, *mut *mut i8) -> i32>,
        _: *mut libc::c_void,
        errmsg: *mut *mut i8,
    ) -> i32;
    fn sqlite3_initialize() -> i32;
    fn sqlite3_config(_: i32, _: ...) -> i32;
    fn sqlite3_db_config(_: *mut sqlite3, op: i32, _: ...) -> i32;
    fn sqlite3_changes64(_: *mut sqlite3) -> i64;
    fn sqlite3_total_changes64(_: *mut sqlite3) -> i64;
    fn sqlite3_interrupt(_: *mut sqlite3);
    fn sqlite3_busy_timeout(_: *mut sqlite3, ms: i32) -> i32;
    fn sqlite3_mprintf(_: *const i8, _: ...) -> *mut i8;
    fn sqlite3_vmprintf(_: *const i8, _: ::std::ffi::VaList) -> *mut i8;
    fn sqlite3_snprintf(_: i32, _: *mut i8, _: *const i8, _: ...) -> *mut i8;
    fn sqlite3_vsnprintf(_: i32, _: *mut i8, _: *const i8, _: ::std::ffi::VaList) -> *mut i8;
    fn sqlite3_malloc(_: i32) -> *mut libc::c_void;
    fn sqlite3_malloc64(_: u64) -> *mut libc::c_void;
    fn sqlite3_realloc(_: *mut libc::c_void, _: i32) -> *mut libc::c_void;
    fn sqlite3_realloc64(_: *mut libc::c_void, _: u64) -> *mut libc::c_void;
    fn sqlite3_free(_: *mut libc::c_void);
    fn sqlite3_randomness(N: i32, P: *mut libc::c_void);
    fn sqlite3_set_authorizer(
        _: *mut sqlite3,
        xAuth: Option<unsafe extern "C" fn(*mut libc::c_void, i32, *const i8, *const i8, *const i8, *const i8) -> i32>,
        pUserData: *mut libc::c_void,
    ) -> i32;
    fn sqlite3_trace_v2(
        _: *mut sqlite3,
        uMask: u32,
        xCallback: Option<unsafe extern "C" fn(u32, *mut libc::c_void, *mut libc::c_void, *mut libc::c_void) -> i32>,
        pCtx: *mut libc::c_void,
    ) -> i32;
    fn sqlite3_progress_handler(_: *mut sqlite3, _: i32, _: Option<unsafe extern "C" fn(*mut libc::c_void) -> i32>, _: *mut libc::c_void);
    fn sqlite3_open(filename: *const i8, ppDb: *mut *mut sqlite3) -> i32;
    fn sqlite3_open_v2(filename: *const i8, ppDb: *mut *mut sqlite3, flags: i32, zVfs: *const i8) -> i32;
    fn sqlite3_errcode(db: *mut sqlite3) -> i32;
    fn sqlite3_extended_errcode(db: *mut sqlite3) -> i32;
    fn sqlite3_errmsg(_: *mut sqlite3) -> *const i8;
    fn sqlite3_error_offset(db: *mut sqlite3) -> i32;
    fn sqlite3_limit(_: *mut sqlite3, id: i32, newVal: i32) -> i32;
    fn sqlite3_prepare_v2(db: *mut sqlite3, zSql: *const i8, nByte: i32, ppStmt: *mut *mut sqlite3_stmt, pzTail: *mut *const i8) -> i32;
    fn sqlite3_sql(pStmt: *mut sqlite3_stmt) -> *const i8;
    fn sqlite3_expanded_sql(pStmt: *mut sqlite3_stmt) -> *mut i8;
    fn sqlite3_stmt_readonly(pStmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_stmt_isexplain(pStmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_bind_blob(
        _: *mut sqlite3_stmt,
        _: i32,
        _: *const libc::c_void,
        n: i32,
        _: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
    ) -> i32;
    fn sqlite3_bind_double(_: *mut sqlite3_stmt, _: i32, _: f64) -> i32;
    fn sqlite3_bind_int(_: *mut sqlite3_stmt, _: i32, _: i32) -> i32;
    fn sqlite3_bind_int64(_: *mut sqlite3_stmt, _: i32, _: i64) -> i32;
    fn sqlite3_bind_null(_: *mut sqlite3_stmt, _: i32) -> i32;
    fn sqlite3_bind_text(
        _: *mut sqlite3_stmt,
        _: i32,
        _: *const i8,
        _: i32,
        _: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
    ) -> i32;
    fn sqlite3_bind_value(_: *mut sqlite3_stmt, _: i32, _: *const sqlite3_value) -> i32;
    fn sqlite3_bind_parameter_count(_: *mut sqlite3_stmt) -> i32;
    fn sqlite3_bind_parameter_name(_: *mut sqlite3_stmt, _: i32) -> *const i8;
    fn sqlite3_column_count(pStmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_column_name(_: *mut sqlite3_stmt, N: i32) -> *const i8;
    fn sqlite3_column_decltype(_: *mut sqlite3_stmt, _: i32) -> *const i8;
    fn sqlite3_step(_: *mut sqlite3_stmt) -> i32;
    fn sqlite3_column_blob(_: *mut sqlite3_stmt, iCol: i32) -> *const libc::c_void;
    fn sqlite3_column_double(_: *mut sqlite3_stmt, iCol: i32) -> f64;
    fn sqlite3_column_int(_: *mut sqlite3_stmt, iCol: i32) -> i32;
    fn sqlite3_column_int64(_: *mut sqlite3_stmt, iCol: i32) -> i64;
    fn sqlite3_column_text(_: *mut sqlite3_stmt, iCol: i32) -> *const u8;
    fn sqlite3_column_value(_: *mut sqlite3_stmt, iCol: i32) -> *mut sqlite3_value;
    fn sqlite3_column_bytes(_: *mut sqlite3_stmt, iCol: i32) -> i32;
    fn sqlite3_column_type(_: *mut sqlite3_stmt, iCol: i32) -> i32;
    fn sqlite3_finalize(pStmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_reset(pStmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_create_function(
        db: *mut sqlite3,
        zFunctionName: *const i8,
        nArg: i32,
        eTextRep: i32,
        pApp: *mut libc::c_void,
        xFunc: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value) -> ()>,
        xStep: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value) -> ()>,
        xFinal: Option<unsafe extern "C" fn(*mut sqlite3_context) -> ()>,
    ) -> i32;
    fn sqlite3_create_window_function(
        db: *mut sqlite3,
        zFunctionName: *const i8,
        nArg: i32,
        eTextRep: i32,
        pApp: *mut libc::c_void,
        xStep: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value) -> ()>,
        xFinal: Option<unsafe extern "C" fn(*mut sqlite3_context) -> ()>,
        xValue: Option<unsafe extern "C" fn(*mut sqlite3_context) -> ()>,
        xInverse: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value) -> ()>,
        xDestroy: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
    ) -> i32;
    fn sqlite3_value_blob(_: *mut sqlite3_value) -> *const libc::c_void;
    fn sqlite3_value_double(_: *mut sqlite3_value) -> f64;
    fn sqlite3_value_int(_: *mut sqlite3_value) -> i32;
    fn sqlite3_value_int64(_: *mut sqlite3_value) -> i64;
    fn sqlite3_value_text(_: *mut sqlite3_value) -> *const u8;
    fn sqlite3_value_bytes(_: *mut sqlite3_value) -> i32;
    fn sqlite3_value_type(_: *mut sqlite3_value) -> i32;
    fn sqlite3_aggregate_context(_: *mut sqlite3_context, nBytes: i32) -> *mut libc::c_void;
    fn sqlite3_user_data(_: *mut sqlite3_context) -> *mut libc::c_void;
    fn sqlite3_context_db_handle(_: *mut sqlite3_context) -> *mut sqlite3;
    fn sqlite3_get_auxdata(_: *mut sqlite3_context, N: i32) -> *mut libc::c_void;
    fn sqlite3_set_auxdata(_: *mut sqlite3_context, N: i32, _: *mut libc::c_void, _: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>);
    fn sqlite3_result_blob(
        _: *mut sqlite3_context,
        _: *const libc::c_void,
        _: i32,
        _: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
    );
    fn sqlite3_result_blob64(
        _: *mut sqlite3_context,
        _: *const libc::c_void,
        _: u64,
        _: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
    );
    fn sqlite3_result_double(_: *mut sqlite3_context, _: f64);
    fn sqlite3_result_error(_: *mut sqlite3_context, _: *const i8, _: i32);
    fn sqlite3_result_error_nomem(_: *mut sqlite3_context);
    fn sqlite3_result_error_code(_: *mut sqlite3_context, _: i32);
    fn sqlite3_result_int(_: *mut sqlite3_context, _: i32);
    fn sqlite3_result_int64(_: *mut sqlite3_context, _: i64);
    fn sqlite3_result_null(_: *mut sqlite3_context);
    fn sqlite3_result_text(_: *mut sqlite3_context, _: *const i8, _: i32, _: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>);
    fn sqlite3_result_text64(
        _: *mut sqlite3_context,
        _: *const i8,
        _: u64,
        _: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
        encoding: u8,
    );
    fn sqlite3_result_value(_: *mut sqlite3_context, _: *mut sqlite3_value);
    fn sqlite3_create_collation(
        _: *mut sqlite3,
        zName: *const i8,
        eTextRep: i32,
        pArg: *mut libc::c_void,
        xCompare: Option<unsafe extern "C" fn(*mut libc::c_void, i32, *const libc::c_void, i32, *const libc::c_void) -> i32>,
    ) -> i32;
    fn sqlite3_sleep(_: i32) -> i32;
    fn sqlite3_get_autocommit(_: *mut sqlite3) -> i32;
    fn sqlite3_db_handle(_: *mut sqlite3_stmt) -> *mut sqlite3;
    fn sqlite3_db_readonly(db: *mut sqlite3, zDbName: *const i8) -> i32;
    fn sqlite3_txn_state(_: *mut sqlite3, zSchema: *const i8) -> i32;
    fn sqlite3_table_column_metadata(
        db: *mut sqlite3,
        zDbName: *const i8,
        zTableName: *const i8,
        zColumnName: *const i8,
        pzDataType: *mut *const i8,
        pzCollSeq: *mut *const i8,
        pNotNull: *mut i32,
        pPrimaryKey: *mut i32,
        pAutoinc: *mut i32,
    ) -> i32;
    fn sqlite3_load_extension(db: *mut sqlite3, zFile: *const i8, zProc: *const i8, pzErrMsg: *mut *mut i8) -> i32;
    fn sqlite3_enable_load_extension(db: *mut sqlite3, onoff: i32) -> i32;
    fn sqlite3_create_module(db: *mut sqlite3, zName: *const i8, p: *const sqlite3_module, pClientData: *mut libc::c_void) -> i32;
    fn sqlite3_declare_vtab(_: *mut sqlite3, zSQL: *const i8) -> i32;
    fn sqlite3_vfs_find(zVfsName: *const i8) -> *mut sqlite3_vfs;
    fn sqlite3_vfs_register(_: *mut sqlite3_vfs, makeDflt: i32) -> i32;
    fn sqlite3_file_control(_: *mut sqlite3, zDbName: *const i8, op: i32, _: *mut libc::c_void) -> i32;
    fn sqlite3_test_control(op: i32, _: ...) -> i32;
    fn sqlite3_keyword_count() -> i32;
    fn sqlite3_keyword_name(_: i32, _: *mut *const i8, _: *mut i32) -> i32;
    fn sqlite3_keyword_check(_: *const i8, _: i32) -> i32;
    fn sqlite3_str_new(_: *mut sqlite3) -> *mut sqlite3_str;
    fn sqlite3_str_finish(_: *mut sqlite3_str) -> *mut i8;
    fn sqlite3_str_appendf(_: *mut sqlite3_str, zFormat: *const i8, _: ...);
    fn sqlite3_str_append(_: *mut sqlite3_str, zIn: *const i8, N: i32);
    fn sqlite3_str_appendall(_: *mut sqlite3_str, zIn: *const i8);
    fn sqlite3_status64(op: i32, pCurrent: *mut i64, pHighwater: *mut i64, resetFlag: i32) -> i32;
    fn sqlite3_db_status(_: *mut sqlite3, op: i32, pCur: *mut i32, pHiwtr: *mut i32, resetFlg: i32) -> i32;
    fn sqlite3_stmt_status(_: *mut sqlite3_stmt, op: i32, resetFlg: i32) -> i32;
    fn sqlite3_backup_init(pDest: *mut sqlite3, zDestName: *const i8, pSource: *mut sqlite3, zSourceName: *const i8)
        -> *mut sqlite3_backup;
    fn sqlite3_backup_step(p: *mut sqlite3_backup, nPage: i32) -> i32;
    fn sqlite3_backup_finish(p: *mut sqlite3_backup) -> i32;
    fn sqlite3_stricmp(_: *const i8, _: *const i8) -> i32;
    fn sqlite3_strnicmp(_: *const i8, _: *const i8, _: i32) -> i32;
    fn sqlite3_strglob(zGlob: *const i8, zStr: *const i8) -> i32;

    fn sqlite3_vtab_config(_: *mut sqlite3, op: i32, _: ...) -> i32;
    fn sqlite3_vtab_collation(_: *mut sqlite3_index_info, _: i32) -> *const i8;
    fn sqlite3_deserialize(db: *mut sqlite3, zSchema: *const i8, pData: *mut u8, szDb: i64, szBuf: i64, mFlags: u32) -> i32;
    fn tolower(_: i32) -> i32;
    fn signal(__sig: i32, __handler: __sighandler_t) -> __sighandler_t;
    fn raise(__sig: i32) -> i32;
    fn isatty(__fd: i32) -> i32;
    fn symlink(__from: *const i8, __to: *const i8) -> i32;
    fn readlink(__path: *const i8, __buf: *mut i8, __len: size_t) -> ssize_t;
    fn unlink(__name: *const i8) -> i32;
    fn opendir(__name: *const i8) -> *mut DIR;
    fn closedir(__dirp: *mut DIR) -> i32;
    fn readdir(__dirp: *mut DIR) -> *mut dirent;
    fn stat(__file: *const i8, __buf: *mut stat) -> i32;
    fn lstat(__file: *const i8, __buf: *mut stat) -> i32;
    fn chmod(__file: *const i8, __mode: __mode_t) -> i32;
    fn utimes(__file: *const i8, __tvp: *const timeval) -> i32;
    fn getrusage(__who: __rusage_who_t, __usage: *mut rusage) -> i32;
    fn time(__timer: *mut time_t) -> time_t;
    fn __errno_location() -> *mut i32;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: u32,
    pub fp_offset: u32,
    pub overflow_arg_area: *mut libc::c_void,
    pub reg_save_area: *mut libc::c_void,
}
pub type size_t = u64;
pub type __dev_t = u64;
pub type __uid_t = u32;
pub type __gid_t = u32;
pub type __ino_t = u64;
pub type __ino64_t = u64;
pub type __mode_t = u32;
pub type __nlink_t = u64;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
pub type __pid_t = i32;
pub type __time_t = libc::c_long;
pub type __suseconds_t = libc::c_long;
pub type __blksize_t = libc::c_long;
pub type __blkcnt_t = libc::c_long;
pub type __ssize_t = libc::c_long;
pub type __syscall_slong_t = libc::c_long;
pub type mode_t = __mode_t;
pub type uid_t = __uid_t;
pub type ssize_t = __ssize_t;
pub type time_t = __time_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}
pub type va_list = __builtin_va_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: i32,
    pub _IO_read_ptr: *mut i8,
    pub _IO_read_end: *mut i8,
    pub _IO_read_base: *mut i8,
    pub _IO_write_base: *mut i8,
    pub _IO_write_ptr: *mut i8,
    pub _IO_write_end: *mut i8,
    pub _IO_buf_base: *mut i8,
    pub _IO_buf_end: *mut i8,
    pub _IO_save_base: *mut i8,
    pub _IO_backup_base: *mut i8,
    pub _IO_save_end: *mut i8,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: i32,
    pub _flags2: i32,
    pub _old_offset: __off_t,
    pub _cur_column: u16,
    pub _vtable_offset: libc::c_schar,
    pub _shortbuf: [i8; 1],
    pub _lock: *mut libc::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut libc::c_void,
    pub __pad5: size_t,
    pub _mode: i32,
    pub _unused2: [i8; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_file {
    pub pMethods: *const sqlite3_io_methods,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_io_methods {
    pub iVersion: i32,
    pub xClose: Option<unsafe extern "C" fn(*mut sqlite3_file) -> i32>,
    pub xRead: Option<unsafe extern "C" fn(*mut sqlite3_file, *mut libc::c_void, i32, i64) -> i32>,
    pub xWrite: Option<unsafe extern "C" fn(*mut sqlite3_file, *const libc::c_void, i32, i64) -> i32>,
    pub xTruncate: Option<unsafe extern "C" fn(*mut sqlite3_file, i64) -> i32>,
    pub xSync: Option<unsafe extern "C" fn(*mut sqlite3_file, i32) -> i32>,
    pub xFileSize: Option<unsafe extern "C" fn(*mut sqlite3_file, *mut i64) -> i32>,
    pub xLock: Option<unsafe extern "C" fn(*mut sqlite3_file, i32) -> i32>,
    pub xUnlock: Option<unsafe extern "C" fn(*mut sqlite3_file, i32) -> i32>,
    pub xCheckReservedLock: Option<unsafe extern "C" fn(*mut sqlite3_file, *mut i32) -> i32>,
    pub xFileControl: Option<unsafe extern "C" fn(*mut sqlite3_file, i32, *mut libc::c_void) -> i32>,
    pub xSectorSize: Option<unsafe extern "C" fn(*mut sqlite3_file) -> i32>,
    pub xDeviceCharacteristics: Option<unsafe extern "C" fn(*mut sqlite3_file) -> i32>,
    pub xShmMap: Option<unsafe extern "C" fn(*mut sqlite3_file, i32, i32, i32, *mut *mut libc::c_void) -> i32>,
    pub xShmLock: Option<unsafe extern "C" fn(*mut sqlite3_file, i32, i32, i32) -> i32>,
    pub xShmBarrier: Option<unsafe extern "C" fn(*mut sqlite3_file) -> ()>,
    pub xShmUnmap: Option<unsafe extern "C" fn(*mut sqlite3_file, i32) -> i32>,
    pub xFetch: Option<unsafe extern "C" fn(*mut sqlite3_file, i64, i32, *mut *mut libc::c_void) -> i32>,
    pub xUnfetch: Option<unsafe extern "C" fn(*mut sqlite3_file, i64, *mut libc::c_void) -> i32>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_vfs {
    pub iVersion: i32,
    pub szOsFile: i32,
    pub mxPathname: i32,
    pub pNext: *mut sqlite3_vfs,
    pub zName: *const i8,
    pub pAppData: *mut libc::c_void,
    pub xOpen: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *const i8, *mut sqlite3_file, i32, *mut i32) -> i32>,
    pub xDelete: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *const i8, i32) -> i32>,
    pub xAccess: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *const i8, i32, *mut i32) -> i32>,
    pub xFullPathname: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *const i8, i32, *mut i8) -> i32>,
    pub xDlOpen: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *const i8) -> *mut libc::c_void>,
    pub xDlError: Option<unsafe extern "C" fn(*mut sqlite3_vfs, i32, *mut i8) -> ()>,
    pub xDlSym: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut libc::c_void, *const i8) -> Option<unsafe extern "C" fn() -> ()>>,
    pub xDlClose: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut libc::c_void) -> ()>,
    pub xRandomness: Option<unsafe extern "C" fn(*mut sqlite3_vfs, i32, *mut i8) -> i32>,
    pub xSleep: Option<unsafe extern "C" fn(*mut sqlite3_vfs, i32) -> i32>,
    pub xCurrentTime: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut f64) -> i32>,
    pub xGetLastError: Option<unsafe extern "C" fn(*mut sqlite3_vfs, i32, *mut i8) -> i32>,
    pub xCurrentTimeInt64: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut i64) -> i32>,
    pub xSetSystemCall: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *const i8, sqlite3_syscall_ptr) -> i32>,
    pub xGetSystemCall: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *const i8) -> sqlite3_syscall_ptr>,
    pub xNextSystemCall: Option<unsafe extern "C" fn(*mut sqlite3_vfs, *const i8) -> *const i8>,
}
pub type sqlite3_syscall_ptr = Option<unsafe extern "C" fn() -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_mem_methods {
    pub xMalloc: Option<unsafe extern "C" fn(i32) -> *mut libc::c_void>,
    pub xFree: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
    pub xRealloc: Option<unsafe extern "C" fn(*mut libc::c_void, i32) -> *mut libc::c_void>,
    pub xSize: Option<unsafe extern "C" fn(*mut libc::c_void) -> i32>,
    pub xRoundup: Option<unsafe extern "C" fn(i32) -> i32>,
    pub xInit: Option<unsafe extern "C" fn(*mut libc::c_void) -> i32>,
    pub xShutdown: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
    pub pAppData: *mut libc::c_void,
}
pub type sqlite3_destructor_type = Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_vtab {
    pub pModule: *const sqlite3_module,
    pub nRef: i32,
    pub zErrMsg: *mut i8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_module {
    pub iVersion: i32,
    pub xCreate:
        Option<unsafe extern "C" fn(*mut sqlite3, *mut libc::c_void, i32, *const *const i8, *mut *mut sqlite3_vtab, *mut *mut i8) -> i32>,
    pub xConnect:
        Option<unsafe extern "C" fn(*mut sqlite3, *mut libc::c_void, i32, *const *const i8, *mut *mut sqlite3_vtab, *mut *mut i8) -> i32>,
    pub xBestIndex: Option<unsafe extern "C" fn(*mut sqlite3_vtab, *mut sqlite3_index_info) -> i32>,
    pub xDisconnect: Option<unsafe extern "C" fn(*mut sqlite3_vtab) -> i32>,
    pub xDestroy: Option<unsafe extern "C" fn(*mut sqlite3_vtab) -> i32>,
    pub xOpen: Option<unsafe extern "C" fn(*mut sqlite3_vtab, *mut *mut sqlite3_vtab_cursor) -> i32>,
    pub xClose: Option<unsafe extern "C" fn(*mut sqlite3_vtab_cursor) -> i32>,
    pub xFilter: Option<unsafe extern "C" fn(*mut sqlite3_vtab_cursor, i32, *const i8, i32, *mut *mut sqlite3_value) -> i32>,
    pub xNext: Option<unsafe extern "C" fn(*mut sqlite3_vtab_cursor) -> i32>,
    pub xEof: Option<unsafe extern "C" fn(*mut sqlite3_vtab_cursor) -> i32>,
    pub xColumn: Option<unsafe extern "C" fn(*mut sqlite3_vtab_cursor, *mut sqlite3_context, i32) -> i32>,
    pub xRowid: Option<unsafe extern "C" fn(*mut sqlite3_vtab_cursor, *mut i64) -> i32>,
    pub xUpdate: Option<unsafe extern "C" fn(*mut sqlite3_vtab, i32, *mut *mut sqlite3_value, *mut i64) -> i32>,
    pub xBegin: Option<unsafe extern "C" fn(*mut sqlite3_vtab) -> i32>,
    pub xSync: Option<unsafe extern "C" fn(*mut sqlite3_vtab) -> i32>,
    pub xCommit: Option<unsafe extern "C" fn(*mut sqlite3_vtab) -> i32>,
    pub xRollback: Option<unsafe extern "C" fn(*mut sqlite3_vtab) -> i32>,
    pub xFindFunction: Option<
        unsafe extern "C" fn(
            *mut sqlite3_vtab,
            i32,
            *const i8,
            *mut Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value) -> ()>,
            *mut *mut libc::c_void,
        ) -> i32,
    >,
    pub xRename: Option<unsafe extern "C" fn(*mut sqlite3_vtab, *const i8) -> i32>,
    pub xSavepoint: Option<unsafe extern "C" fn(*mut sqlite3_vtab, i32) -> i32>,
    pub xRelease: Option<unsafe extern "C" fn(*mut sqlite3_vtab, i32) -> i32>,
    pub xRollbackTo: Option<unsafe extern "C" fn(*mut sqlite3_vtab, i32) -> i32>,
    pub xShadowName: Option<unsafe extern "C" fn(*const i8) -> i32>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_vtab_cursor {
    pub pVtab: *mut sqlite3_vtab,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_index_info {
    pub nConstraint: i32,
    pub aConstraint: *mut sqlite3_index_constraint,
    pub nOrderBy: i32,
    pub aOrderBy: *mut sqlite3_index_orderby,
    pub aConstraintUsage: *mut sqlite3_index_constraint_usage,
    pub idxNum: i32,
    pub idxStr: *mut i8,
    pub needToFreeIdxStr: i32,
    pub orderByConsumed: i32,
    pub estimatedCost: f64,
    pub estimatedRows: i64,
    pub idxFlags: i32,
    pub colUsed: u64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_index_constraint_usage {
    pub argvIndex: i32,
    pub omit: u8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_index_orderby {
    pub iColumn: i32,
    pub desc: u8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3_index_constraint {
    pub iColumn: i32,
    pub op: u8,
    pub usable: u8,
    pub iTermOffset: i32,
}

type C2RustUnnamed = u32;
pub type __sighandler_t = Option<unsafe extern "C" fn(i32) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dirent {
    pub d_ino: __ino64_t,
    pub d_off: __off64_t,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [i8; 256],
}
pub type DIR = __dirstream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stat {
    pub st_dev: __dev_t,
    pub st_ino: __ino_t,
    pub st_nlink: __nlink_t,
    pub st_mode: __mode_t,
    pub st_uid: __uid_t,
    pub st_gid: __gid_t,
    pub __pad0: i32,
    pub st_rdev: __dev_t,
    pub st_size: __off_t,
    pub st_blksize: __blksize_t,
    pub st_blocks: __blkcnt_t,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    pub __glibc_reserved: [__syscall_slong_t; 3],
}
pub type __rusage_who = i32;
pub const RUSAGE_CHILDREN: __rusage_who = -1;
pub const RUSAGE_SELF: __rusage_who = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct rusage {
    pub ru_utime: timeval,
    pub ru_stime: timeval,
    pub c2rust_unnamed: C2RustUnnamed_13,
    pub c2rust_unnamed_0: C2RustUnnamed_12,
    pub c2rust_unnamed_1: C2RustUnnamed_11,
    pub c2rust_unnamed_2: C2RustUnnamed_10,
    pub c2rust_unnamed_3: C2RustUnnamed_9,
    pub c2rust_unnamed_4: C2RustUnnamed_8,
    pub c2rust_unnamed_5: C2RustUnnamed_7,
    pub c2rust_unnamed_6: C2RustUnnamed_6,
    pub c2rust_unnamed_7: C2RustUnnamed_5,
    pub c2rust_unnamed_8: C2RustUnnamed_4,
    pub c2rust_unnamed_9: C2RustUnnamed_3,
    pub c2rust_unnamed_10: C2RustUnnamed_2,
    pub c2rust_unnamed_11: C2RustUnnamed_1,
    pub c2rust_unnamed_12: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub ru_nivcsw: libc::c_long,
    pub __ru_nivcsw_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub ru_nvcsw: libc::c_long,
    pub __ru_nvcsw_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub ru_nsignals: libc::c_long,
    pub __ru_nsignals_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_3 {
    pub ru_msgrcv: libc::c_long,
    pub __ru_msgrcv_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_4 {
    pub ru_msgsnd: libc::c_long,
    pub __ru_msgsnd_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_5 {
    pub ru_oublock: libc::c_long,
    pub __ru_oublock_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_6 {
    pub ru_inblock: libc::c_long,
    pub __ru_inblock_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_7 {
    pub ru_nswap: libc::c_long,
    pub __ru_nswap_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_8 {
    pub ru_majflt: libc::c_long,
    pub __ru_majflt_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_9 {
    pub ru_minflt: libc::c_long,
    pub __ru_minflt_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_10 {
    pub ru_isrss: libc::c_long,
    pub __ru_isrss_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_11 {
    pub ru_idrss: libc::c_long,
    pub __ru_idrss_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub ru_ixrss: libc::c_long,
    pub __ru_ixrss_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_13 {
    pub ru_maxrss: libc::c_long,
    pub __ru_maxrss_word: __syscall_slong_t,
}
pub type __rusage_who_t = i32;

#[derive(Clone)]
struct ShellText {
    inner: String,
}

impl Deref for ShellText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        return &self.inner;
    }
}

impl fmt::Display for ShellText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl ShellText {
    pub fn new() -> Self {
        Self { inner: String::new() }
    }

    pub fn new_with_capacity(size: usize) -> Self {
        Self {
            inner: String::with_capacity(size),
        }
    }

    pub fn push(&mut self, text: &str) {
        self.append(text, b'\0');
    }

    pub fn push_u8(&mut self, text: u8) {
        self.inner.push(text as char);
    }

    pub fn append(&mut self, text: &str, quote: u8) {
        if quote == b'\0' {
            self.inner.push_str(text);
        } else {
            self.inner.push(quote as char);
            let text = text.replace(quote as char, &String::from_utf8(vec![quote; 2]).unwrap());
            self.inner.push_str(&text);
            self.inner.push(quote as char);
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn to_string(self) -> String {
        self.inner
    }
    pub fn as_str(&self) -> &str {
        &self.inner
    }
    // pub unsafe fn as_c_str(&self) -> *const i8 {
    //     to_cstring!(self.inner.as_str()) as *const u8 as *const i8
    // }

    pub fn to_cstr(&mut self) -> *const i8 {
        self.inner.push('\0');
        self.inner.as_ptr() as _
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SHA3Context {
    pub u: C2RustUnnamed_15,
    pub nRate: u32,
    pub nLoaded: u32,
    pub ixMask: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_15 {
    pub s: [u64; 25],
    pub x: [u8; 1600],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fsdir_cursor {
    pub base: sqlite3_vtab_cursor,
    pub nLvl: i32,
    pub iLvl: i32,
    pub aLvl: *mut FsdirLevel,
    pub zBase: *const i8,
    pub nBase: i32,
    pub sStat: stat,
    pub zPath: *mut i8,
    pub iRowid: i64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FsdirLevel {
    pub pDir: *mut DIR,
    pub zDir: *mut i8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fsdir_tab {
    pub base: sqlite3_vtab,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct completion_vtab {
    pub base: sqlite3_vtab,
    pub db: *mut sqlite3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct completion_cursor {
    pub base: sqlite3_vtab_cursor,
    pub db: *mut sqlite3,
    pub nPrefix: i32,
    pub nLine: i32,
    pub zPrefix: *mut i8,
    pub zLine: *mut i8,
    pub zCurrentRow: *const i8,
    pub szRow: i32,
    pub pStmt: *mut sqlite3_stmt,
    pub iRowid: i64,
    pub ePhase: i32,
    pub j: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ApndFile {
    pub base: sqlite3_file,
    pub iPgOne: i64,
    pub iMark: i64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Decimal {
    pub sign: i8,
    pub oom: i8,
    pub isNull: i8,
    pub isInit: i8,
    pub nDigit: i32,
    pub nFrac: i32,
    pub a: *mut libc::c_schar,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_16 {
    pub zFuncName: *const i8,
    pub nArg: i32,
    pub xFunc: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_17 {
    pub zFName: *mut i8,
    pub nArg: i32,
    pub iAux: i32,
    pub xFunc: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct series_cursor {
    pub base: sqlite3_vtab_cursor,
    pub isDesc: i32,
    pub iRowid: i64,
    pub iValue: i64,
    pub mnValue: i64,
    pub mxValue: i64,
    pub iStep: i64,
}
pub type ReStateNumber = u16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ReStateSet {
    pub nState: u32,
    pub aState: *mut ReStateNumber,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ReInput {
    pub z: *const u8,
    pub i: i32,
    pub mx: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ReCompiled {
    pub sIn: ReInput,
    pub zErr: *const i8,
    pub aOp: *mut i8,
    pub aArg: *mut i32,
    pub xNextChar: Option<unsafe extern "C" fn(*mut ReInput) -> u32>,
    pub zInit: [u8; 12],
    pub nInit: i32,
    pub nState: u32,
    pub nAlloc: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sqlite3expert {
    pub iSample: i32,
    pub db: *mut sqlite3,
    pub dbm: *mut sqlite3,
    pub dbv: *mut sqlite3,
    pub pTable: *mut IdxTable,
    pub pScan: *mut IdxScan,
    pub pWrite: *mut IdxWrite,
    pub pStatement: *mut IdxStatement,
    pub bRun: i32,
    pub pzErrmsg: *mut *mut i8,
    pub rc: i32,
    pub hIdx: IdxHash,
    pub zCandidates: *mut i8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxHash {
    pub pFirst: *mut IdxHashEntry,
    pub aHash: [*mut IdxHashEntry; 1023],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxHashEntry {
    pub zKey: *mut i8,
    pub zVal: *mut i8,
    pub zVal2: *mut i8,
    pub pHashNext: *mut IdxHashEntry,
    pub pNext: *mut IdxHashEntry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxStatement {
    pub iId: i32,
    pub zSql: *mut i8,
    pub zIdx: *mut i8,
    pub zEQP: *mut i8,
    pub pNext: *mut IdxStatement,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxWrite {
    pub pTab: *mut IdxTable,
    pub eOp: i32,
    pub pNext: *mut IdxWrite,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxTable {
    pub nCol: i32,
    pub zName: *mut i8,
    pub aCol: *mut IdxColumn,
    pub pNext: *mut IdxTable,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxColumn {
    pub zName: *mut i8,
    pub zColl: *mut i8,
    pub iPk: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxScan {
    pub pTab: *mut IdxTable,
    pub iDb: i32,
    pub covering: i64,
    pub pOrder: *mut IdxConstraint,
    pub pEq: *mut IdxConstraint,
    pub pRange: *mut IdxConstraint,
    pub pNextScan: *mut IdxScan,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxConstraint {
    pub zColl: *mut i8,
    pub bRange: i32,
    pub iCol: i32,
    pub bFlag: i32,
    pub bDesc: i32,
    pub pNext: *mut IdxConstraint,
    pub pLink: *mut IdxConstraint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExpertCsr {
    pub base: sqlite3_vtab_cursor,
    pub pData: *mut sqlite3_stmt,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExpertVtab {
    pub base: sqlite3_vtab,
    pub pTab: *mut IdxTable,
    pub pExpert: *mut sqlite3expert,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxRemCtx {
    pub nSlot: i32,
    pub aSlot: [IdxRemSlot; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxRemSlot {
    pub eType: i32,
    pub iVal: i64,
    pub rVal: f64,
    pub nByte: i32,
    pub n: i32,
    pub z: *mut i8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IdxSampleCtx {
    pub iTarget: i32,
    pub target: f64,
    pub nRow: f64,
    pub nRet: f64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExpertInfo {
    pub pExpert: *mut sqlite3expert,
    pub bVerbose: i32,
}

#[derive(Clone)]
pub struct EQPGraphRow {
    pub iEqpId: i32,
    pub iParentId: i32,
    // pub pNext: *mut EQPGraphRow,
    pub zText: String,
}

impl EQPGraphRow {
    pub fn new(id: i32, parentId: i32, zText: String) -> Self {
        Self {
            iEqpId: id,
            iParentId: parentId,
            zText: zText,
        }
    }
}

/* All EQP output is collected into an instance of the following */
#[derive(Clone)]
pub struct EQPGraph {
    pub inner: Vec<EQPGraphRow>,
    // pub zPrefix: String,
}

impl EQPGraph {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            // zPrefix: String::with_capacity(100),
        }
    }

    pub fn last(&self) -> Option<&EQPGraphRow> {
        self.inner.last()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn append(&mut self, item: EQPGraphRow) {
        self.inner.push(item);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColModeOpts {
    pub iWrap: i32,
    pub bQuote: u8,
    pub bWordWrap: u8,
}

impl ColModeOpts {
    pub fn new(iWrap: i32, bQuote: u8, bWordWrap: u8) -> Self {
        Self { iWrap, bQuote, bWordWrap }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct ShellState {
    pub db: *mut sqlite3,
    pub autoExplain: u8,
    pub autoEQP: u8,
    pub autoEQPtest: u8,
    pub autoEQPtrace: u8,
    pub scanstatsOn: u8,
    pub openMode: u8,
    pub doXdgOpen: u8,
    pub nEqpLevel: u8,
    pub eTraceType: u8,
    pub bSafeMode: u8,
    pub bSafeModePersist: u8,
    pub cmOpts: ColModeOpts,
    pub statsOn: u32,
    pub mEqpLines: u32,
    pub inputNesting: i32,
    pub outCount: i32,
    pub cnt: i32,
    pub lineno: i32,
    pub openFlags: i32,
    pub traceOut: *mut FILE,
    pub nErr: i32,
    pub mode: i32,
    pub modePrior: i32,
    pub cMode: i32,
    pub normalMode: i32,
    pub writableSchema: i32,
    pub showHeader: i32,
    pub nCheck: i32,
    pub nProgress: u32,
    pub mxProgress: u32,
    pub flgProgress: u32,
    pub shellFlgs: u32,
    pub priorShFlgs: u32,
    pub szMax: i64,
    pub zDestTable: String,
    pub zTempFile: String,
    pub zTestcase: SmolStr,
    pub colSeparator: SmolStr,
    pub rowSeparator: SmolStr,
    pub colSepPrior: SmolStr,
    pub rowSepPrior: SmolStr,
    pub colWidth: Vec<i32>,
    pub actualWidth: Vec<i32>,
    pub nWidth: i32,
    pub nullValue: SmolStr,
    pub outfile: [i8; 4096],
    pub pStmt: *mut sqlite3_stmt,
    pub pLog: *mut FILE,
    pub aAuxDb: Vec<AuxDb>,
    pub pAuxDb: *mut AuxDb,
    pub aiIndent: Vec<i32>,
    pub nIndent: i32,
    pub iIndent: i32,
    pub zNonce: String,
    pub sGraph: EQPGraph,
    pub expert: ExpertInfo,

    pub mainPrompt: SmolStr,
    pub continuePrompt: SmolStr,
}

impl Default for ShellState {
    fn default() -> Self {
        let auxDbs = vec![
            AuxDb {
                db: 0 as *mut sqlite3,
                zDbFilename: String::new(),
            },
            AuxDb {
                db: 0 as *mut sqlite3,
                zDbFilename: String::new(),
            },
            AuxDb {
                db: 0 as *mut sqlite3,
                zDbFilename: String::new(),
            },
            AuxDb {
                db: 0 as *mut sqlite3,
                zDbFilename: String::new(),
            },
            AuxDb {
                db: 0 as *mut sqlite3,
                zDbFilename: String::new(),
            },
        ];

        ShellState {
            db: 0 as *mut sqlite3,
            autoExplain: 0,
            autoEQP: 0,
            autoEQPtest: 0,
            autoEQPtrace: 0,
            scanstatsOn: 0,
            openMode: 0,
            doXdgOpen: 0,
            nEqpLevel: 0,
            eTraceType: 0,
            bSafeMode: 0,
            bSafeModePersist: 0,
            cmOpts: ColModeOpts::new(0, 0, 0),
            statsOn: 0,
            mEqpLines: 0,
            inputNesting: 0,
            outCount: 0,
            cnt: 0,
            lineno: 0,
            openFlags: 0,
            traceOut: 0 as *mut FILE,
            nErr: 0,
            mode: 0,
            modePrior: 0,
            cMode: 0,
            normalMode: 0,
            writableSchema: 0,
            showHeader: 0,
            nCheck: 0,
            nProgress: 0,
            mxProgress: 0,
            flgProgress: 0,
            shellFlgs: 0,
            priorShFlgs: 0,
            szMax: 0,
            zDestTable: String::new(),
            zTempFile: String::new(),
            zTestcase: SmolStr::new_inline(""),
            colSeparator: SmolStr::new_inline(""),
            rowSeparator: SmolStr::new_inline(""),
            colSepPrior: SmolStr::new_inline(""),
            rowSepPrior: SmolStr::new_inline(""),
            colWidth: vec![],
            actualWidth: vec![],
            nWidth: 0,
            nullValue: SmolStr::new_inline(""),
            outfile: [0; 4096],
            pStmt: 0 as *mut sqlite3_stmt,
            pLog: 0 as *mut FILE,
            aAuxDb: auxDbs,
            pAuxDb: 0 as *mut AuxDb,
            aiIndent: Vec::new(),
            nIndent: 0,
            iIndent: 0,
            zNonce: String::new(),
            sGraph: EQPGraph::new(),
            expert: ExpertInfo {
                pExpert: 0 as *mut sqlite3expert,
                bVerbose: 0,
            },

            mainPrompt: SmolStr::new_inline(""),
            continuePrompt: SmolStr::new_inline(""),
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct AuxDb {
    pub db: *mut sqlite3,
    pub zDbFilename: String,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_18 {
    pub zPattern: &'static str,
    pub zDesc: &'static str,
}
pub type QuickScanState = u32;
pub const QSS_Start: QuickScanState = 0;
pub const QSS_ScanMask: QuickScanState = 768;
pub const QSS_CharMask: QuickScanState = 255;
pub const QSS_EndingSemi: QuickScanState = 512;
pub const QSS_HasDark: QuickScanState = 256;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_19 {
    pub zCtrlName: &'static str,
    pub ctrlCode: i32,
    pub unSafe: i32,
    pub zUsage: &'static str,
}
#[derive(Clone)]
#[repr(C)]
pub struct C2RustUnnamed_20 {
    pub zLimitName: &'static str,
    pub limitCode: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImportCtx {
    pub zFile: *const i8,
    pub in_0: *mut FILE,
    pub xCloser: Option<unsafe extern "C" fn(*mut FILE) -> i32>,
    pub z: *mut i8,
    pub n: i32,
    pub nAlloc: i32,
    pub nLine: i32,
    pub nRow: i32,
    pub nErr: i32,
    pub bNotFirst: i32,
    pub cTerm: i32,
    pub cColSep: i32,
    pub cRowSep: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_21 {
    pub zCtrlName: &'static str,
    pub ctrlCode: i32,
    pub zUsage: &'static str,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
    pub zName: &'static str,
    pub zSql: &'static str,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
    pub zName: &'static str,
    pub ofst: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DbConfigChoices {
    pub zName: &'static str,
    pub op: i32,
}
static mut enableTimer: i32 = 0;

unsafe fn timeOfDay() -> i64 {
    static mut clockVfs: *mut sqlite3_vfs = std::ptr::null_mut();
    let mut t: i64 = 0;
    if clockVfs.is_null() {
        clockVfs = sqlite3_vfs_find(0 as *const i8);
    }
    if clockVfs.is_null() {
        return 0;
    }
    if (*clockVfs).iVersion >= 2 && ((*clockVfs).xCurrentTimeInt64).is_some() {
        ((*clockVfs).xCurrentTimeInt64).expect("non-null function pointer")(clockVfs, &mut t);
    } else {
        let mut r: f64 = 0.;
        ((*clockVfs).xCurrentTime).expect("non-null function pointer")(clockVfs, &mut r);
        t = (r * 86400000.0f64) as i64;
    }
    return t;
}

unsafe fn beginTimer() -> (i64, rusage) {
    let mut sBegin: rusage = rusage {
        ru_utime: timeval { tv_sec: 0, tv_usec: 0 },
        ru_stime: timeval { tv_sec: 0, tv_usec: 0 },
        c2rust_unnamed: C2RustUnnamed_13 { ru_maxrss: 0 },
        c2rust_unnamed_0: C2RustUnnamed_12 { ru_ixrss: 0 },
        c2rust_unnamed_1: C2RustUnnamed_11 { ru_idrss: 0 },
        c2rust_unnamed_2: C2RustUnnamed_10 { ru_isrss: 0 },
        c2rust_unnamed_3: C2RustUnnamed_9 { ru_minflt: 0 },
        c2rust_unnamed_4: C2RustUnnamed_8 { ru_majflt: 0 },
        c2rust_unnamed_5: C2RustUnnamed_7 { ru_nswap: 0 },
        c2rust_unnamed_6: C2RustUnnamed_6 { ru_inblock: 0 },
        c2rust_unnamed_7: C2RustUnnamed_5 { ru_oublock: 0 },
        c2rust_unnamed_8: C2RustUnnamed_4 { ru_msgsnd: 0 },
        c2rust_unnamed_9: C2RustUnnamed_3 { ru_msgrcv: 0 },
        c2rust_unnamed_10: C2RustUnnamed_2 { ru_nsignals: 0 },
        c2rust_unnamed_11: C2RustUnnamed_1 { ru_nvcsw: 0 },
        c2rust_unnamed_12: C2RustUnnamed_0 { ru_nivcsw: 0 },
    };
    let mut iBegin: i64 = 0;
    if enableTimer != 0 {
        getrusage(RUSAGE_SELF as i32, &mut sBegin);
        iBegin = timeOfDay();
    }
    (iBegin, sBegin)
}

fn timeDiff(pStart: &timeval, pEnd: &timeval) -> f64 {
    return (pEnd.tv_usec - pStart.tv_usec) as f64 * 0.000001f64 + (pEnd.tv_sec - pStart.tv_sec) as f64;
}

unsafe fn endTimer(iBegin: i64, sBegin: rusage) {
    if enableTimer == 0 {
        return;
    }

    let iEnd: i64 = timeOfDay();
    let mut sEnd: rusage = rusage {
        ru_utime: timeval { tv_sec: 0, tv_usec: 0 },
        ru_stime: timeval { tv_sec: 0, tv_usec: 0 },
        c2rust_unnamed: C2RustUnnamed_13 { ru_maxrss: 0 },
        c2rust_unnamed_0: C2RustUnnamed_12 { ru_ixrss: 0 },
        c2rust_unnamed_1: C2RustUnnamed_11 { ru_idrss: 0 },
        c2rust_unnamed_2: C2RustUnnamed_10 { ru_isrss: 0 },
        c2rust_unnamed_3: C2RustUnnamed_9 { ru_minflt: 0 },
        c2rust_unnamed_4: C2RustUnnamed_8 { ru_majflt: 0 },
        c2rust_unnamed_5: C2RustUnnamed_7 { ru_nswap: 0 },
        c2rust_unnamed_6: C2RustUnnamed_6 { ru_inblock: 0 },
        c2rust_unnamed_7: C2RustUnnamed_5 { ru_oublock: 0 },
        c2rust_unnamed_8: C2RustUnnamed_4 { ru_msgsnd: 0 },
        c2rust_unnamed_9: C2RustUnnamed_3 { ru_msgrcv: 0 },
        c2rust_unnamed_10: C2RustUnnamed_2 { ru_nsignals: 0 },
        c2rust_unnamed_11: C2RustUnnamed_1 { ru_nvcsw: 0 },
        c2rust_unnamed_12: C2RustUnnamed_0 { ru_nivcsw: 0 },
    };
    getrusage(RUSAGE_SELF as i32, &mut sEnd);
    println!(
        "Run Time: real {} user {} sys {}",
        (iEnd - iBegin) as f64 * 0.001f64,
        timeDiff(&sBegin.ru_utime, &sEnd.ru_utime),
        timeDiff(&sBegin.ru_stime, &sEnd.ru_stime),
    );
}

static mut bail_on_error: bool = false;
static mut stdin_is_interactive: bool = true;
static mut stdout_is_console: i32 = 1;
static mut globalDb: *mut sqlite3 = 0 as *const sqlite3 as *mut sqlite3;
static mut seenInterrupt: i32 = 0;

/// Indicate out-of-memory and exit.
fn shell_out_of_memory() -> ! {
    eprintln!("Error: out of memory");
    std::process::exit(1);
}

/// Check a pointer to see if it is NULL.  If it is NULL, exit with an out-of-memory error.
fn shell_check_oom(p: *const libc::c_void) {
    if p.is_null() {
        shell_out_of_memory();
    }
}

/*
** Output string zUtf to stream pOut as w characters.  If w is negative,
** then right-justify the text.  W is the width in UTF-8 characters, not
** in bytes.  This is different from the %*.*s specification in printf
** since with %*.*s the width is measured in bytes, not characters.
*/
fn utf8_width_print(w: i32, zUtf: &str) {
    let aw: i32 = if w < 0 { -w } else { w };

    let len = w.abs() as usize;
    let s: String = zUtf.chars().take(len).collect();

    print!("{content:>width$}", width = w as usize, content = s);
}

/// Compute a string length that is limited to what can be stored in lower 30 bits of a 32-bit signed integer.
unsafe fn strlen30(z: *const i8) -> i32 {
    let mut z2: *const i8 = z;
    while *z2 != 0 {
        z2 = z2.offset(1);
    }
    return 0x3fffffff as i32 & z2.offset_from(z) as libc::c_long as i32;
}

/// Return the length of a string in characters.  Multibyte UTF8 characters count as a single character.
unsafe fn strlenChar(mut z: *const i8) -> i32 {
    let mut n = 0;
    while *z != 0 {
        if 0xc0 & *z as i32 != 0x80 {
            n += 1;
        }
        z = z.offset(1);
    }
    return n;
}

/// Return open FILE * if zFile exists, can be opened for read
/// and is an ordinary file or a character stream source. Otherwise return 0.
unsafe fn openChrSource(mut zFile: *const i8) -> *mut FILE {
    let mut x: stat = {
        let mut init = stat {
            st_dev: 0 as __dev_t,
            st_ino: 0,
            st_nlink: 0,
            st_mode: 0,
            st_uid: 0,
            st_gid: 0,
            __pad0: 0,
            st_rdev: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_atim: timespec { tv_sec: 0, tv_nsec: 0 },
            st_mtim: timespec { tv_sec: 0, tv_nsec: 0 },
            st_ctim: timespec { tv_sec: 0, tv_nsec: 0 },
            __glibc_reserved: [0; 3],
        };
        init
    };
    let mut rc: i32 = stat(zFile, &mut x);
    if rc != 0 {
        return 0 as *mut FILE;
    }
    if x.st_mode & 0o170000 == 0o100000 || x.st_mode & 0o170000 == 0o10000 || x.st_mode & 0o170000 == 0o20000 {
        return fopen(zFile, b"rb\0" as *const u8 as *const i8);
    } else {
        return 0 as *mut FILE;
    };
}

/// Return the value of a hexadecimal digit.  Return -1 if the input is not a hex digit.
fn hexDigitValue(c: i8) -> i32 {
    let c = c as u8;
    match c {
        b'0'..=b'9' => return (c - b'0') as i32,
        b'a'..=b'f' => return (c - b'a' + 10) as i32,
        b'A'..=b'F' => return (c - b'A' + 10) as i32,
        _ => return -1,
    }
}

fn integerValue(zArg: &str) -> i64 {
    let zArg = zArg.as_bytes();

    const aMult: [(&[u8], i64); 9] = [
        (b"KiB", 1024),
        (b"MiB", 1024 * 1024),
        (b"GiB", 1024 * 1024 * 1024),
        (b"KB", 1000),
        (b"MB", 1000000),
        (b"GB", 1000000000),
        (b"K", 1000),
        (b"M", 1000000),
        (b"G", 1000000000),
    ];

    let mut isNeg = false;
    let mut offset: usize = 0;

    if zArg[offset] == b'-' {
        isNeg = true;
        offset += 1;
    } else if zArg[offset] == b'+' {
        offset += 1;
    }

    let mut v: i64 = 0;
    if zArg[offset] == b'0' && zArg[offset + 1] == b'x' {
        let mut x: i32 = 0;
        offset += 2;

        loop {
            x = hexDigitValue(zArg[offset] as i8);
            if !(x >= 0) {
                break;
            }
            v = (v << 4) + x as i64;
            offset += 1;
        }
    } else {
        while offset < zArg.len() && zArg[offset].is_ascii_digit() {
            v = v * 10 as i64 + zArg[offset] as i64 - '0' as i64;
            offset += 1;
        }
    }

    for (zSuffix, iMult) in aMult {
        if zArg[offset..].starts_with(zSuffix) {
            v *= iMult;
            break;
        }
    }

    return if isNeg { -v } else { v };
}

// unsafe extern "C" fn initText(mut p: *mut ShellText) {
//     memset(p as *mut libc::c_void, 0, ::std::mem::size_of::<ShellText>() as u64);
// }
// unsafe extern "C" fn freeText(mut p: *mut ShellText) {
//     free((*p).z as *mut libc::c_void);
//     initText(p);
// }

/*
** Attempt to determine if identifier zName needs to be quoted, either
** because it contains non-alphanumeric characters, or because it is an
** SQLite keyword.  Be conservative in this estimate:  When in doubt assume
** that quoting is required.
**
** Return '"' if quoting is required.  Return 0 if no quoting is required.
*/
fn quoteChar(zName: &str) -> u8 {
    for (index, ch) in zName.char_indices() {
        if index == 0 && (ch.is_ascii_alphabetic() == false && ch != '_') {
            return b'"';
        }

        if ch.is_ascii_alphanumeric() == false && ch != '_' {
            return b'"';
        }
    }

    if unsafe { sqlite3_keyword_check(to_cstring!(zName), zName.len() as i32) } != 0 {
        return b'"';
    }

    return b'\0';
}

unsafe fn shellFakeSchema(mut db: *mut sqlite3, zSchema: &str, zName: &str) -> String {
    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut zSql: *mut i8 = std::ptr::null_mut();

    let mut cQuote: u8 = 0;
    let mut zDiv = "(";
    let mut nRow: i32 = 0;
    zSql = sqlite3_mprintf(
        b"PRAGMA \"%w\".table_info=%Q;\0" as *const u8 as *const i8,
        if !zSchema.is_empty() {
            to_cstring!(zSchema)
        } else {
            b"main\0" as *const u8 as *const i8
        },
        zName,
    );
    shell_check_oom(zSql as *mut libc::c_void);
    sqlite3_prepare_v2(db, zSql, -1, &mut pStmt, 0 as *mut *const i8);
    sqlite3_free(zSql as *mut libc::c_void);

    let mut s: ShellText = ShellText::new();
    if !zSchema.is_empty() {
        cQuote = quoteChar(&zSchema);
        if cQuote != 0 && zSchema.eq_ignore_ascii_case("temp") {
            cQuote = 0;
        }
        s.append(&zSchema, cQuote);
        s.append(".", 0);
    }

    cQuote = quoteChar(zName);
    s.append(zName, cQuote);
    while sqlite3_step(pStmt) == 100 {
        let mut zCol = cstr_to_string!(sqlite3_column_text(pStmt, 1));
        if zCol.is_empty() {
            zCol = Cow::from("");
        }

        nRow += 1;
        s.append(zDiv, 0);
        zDiv = ",";

        cQuote = quoteChar(&zCol);
        s.append(&zCol, cQuote);
    }
    s.append(")", 0);
    sqlite3_finalize(pStmt);
    if nRow == 0 {
        s.clear();
    }
    return s.to_string();
}

unsafe extern "C" fn shellModuleSchema(mut pCtx: *mut sqlite3_context, mut nVal: i32, mut apVal: *mut *mut sqlite3_value) {
    let mut zName: *const i8 = 0 as *const i8;

    zName = sqlite3_value_text(*apVal.offset(0)) as *const i8;
    let mut zFake = if !zName.is_null() {
        shellFakeSchema(sqlite3_context_db_handle(pCtx), "", &cstr_to_string!(zName))
    } else {
        String::new()
    };

    if !zFake.is_empty() {
        sqlite3_result_text(pCtx, to_cstring!(format!("/* {} */", zFake)), -1, Some(sqlite3_free));
    }
}

unsafe extern "C" fn shellAddSchemaName(mut pCtx: *mut sqlite3_context, mut nVal: i32, mut apVal: *mut *mut sqlite3_value) {
    const aPrefix: [&'static str; 6] = ["TABLE", "INDEX", "UNIQUE INDEX", "VIEW", "TRIGGER", "VIRTUAL TABLE"];
    let mut zIn: *const i8 = sqlite3_value_text(*apVal.offset(0)) as *const i8;
    let mut zSchema: *const i8 = sqlite3_value_text(*apVal.offset(1)) as *const i8;
    let mut zName: *const i8 = sqlite3_value_text(*apVal.offset(2 as isize)) as *const i8;
    let mut db: *mut sqlite3 = sqlite3_context_db_handle(pCtx);

    let zIn = cstr_to_string!(zIn);
    let zSchema = cstr_to_string!(zSchema);
    let zName = cstr_to_string!(zName);

    if zIn == "CREATE " {
        for prefix in aPrefix {
            let n = prefix.len();
            if zIn[7..].starts_with(prefix) && zIn.as_bytes()[n + 7] == b' ' {
                let mut z = String::new();
                let mut zFake = String::new();
                if zSchema.is_empty() == false {
                    let cQuote = quoteChar(&zSchema);
                    if cQuote != 0 && zSchema.eq_ignore_ascii_case("temp") == false {
                        let mut text = ShellText::new();
                        text.append(&zSchema, b'"');

                        z = format!(
                            "{content1:width1$} \"{content2}\".{content3}",
                            width1 = (n + 7) as usize,
                            content1 = zIn,
                            content2 = text,
                            content3 = &zIn[(n + 8)..],
                        );
                    } else {
                        z = format!(
                            "{content1:width1$} {content2}.{content3}",
                            width1 = (n + 7) as usize,
                            content1 = zIn,
                            content2 = zSchema,
                            content3 = &zIn[(n + 8)..],
                        );
                    }
                }
                if zName.is_empty() == false && prefix.starts_with('V') && {
                    zFake = shellFakeSchema(db, &zSchema, &zName);
                    zFake.is_empty() == false
                } {
                    if z.is_empty() {
                        z = format!("{}\n/* {} */", zIn, &zFake);
                    } else {
                        z = format!("{}\n/* {} */", z, &zFake);
                    }
                    zFake.clear();
                }
                if z.is_empty() == false {
                    sqlite3_result_text(pCtx, to_cstring!(z), -1, Some(sqlite3_free));
                    return;
                }
            }
        }
    }
    sqlite3_result_value(pCtx, *apVal.offset(0));
}
unsafe extern "C" fn KeccakF1600Step(mut p: *mut SHA3Context) {
    let mut i: i32 = 0;
    let mut b0: u64 = 0;
    let mut b1: u64 = 0;
    let mut b2: u64 = 0;
    let mut b3: u64 = 0;
    let mut b4: u64 = 0;
    let mut c0: u64 = 0;
    let mut c1: u64 = 0;
    let mut c2: u64 = 0;
    let mut c3: u64 = 0;
    let mut c4: u64 = 0;
    let mut d0: u64 = 0;
    let mut d1: u64 = 0;
    let mut d2: u64 = 0;
    let mut d3: u64 = 0;
    let mut d4: u64 = 0;
    const RC: [u64; 24] = [
        0x1 as u64,
        0x8082 as u64,
        0x800000000000808a as u64,
        0x8000000080008000 as u64,
        0x808b as u64,
        0x80000001 as u64,
        0x8000000080008081 as u64,
        0x8000000000008009 as u64,
        0x8a as u64,
        0x88 as u64,
        0x80008009 as u64,
        0x8000000a as u64,
        0x8000808b as u64,
        0x800000000000008b as u64,
        0x8000000000008089 as u64,
        0x8000000000008003 as u64,
        0x8000000000008002 as u64,
        0x8000000000000080 as u64,
        0x800a as u64,
        0x800000008000000a as u64,
        0x8000000080008081 as u64,
        0x8000000000008080 as u64,
        0x80000001 as u64,
        0x8000000080008008 as u64,
    ];
    for i in (0..24).step_by(4) {
        c0 = (*p).u.s[0] ^ (*p).u.s[5] ^ (*p).u.s[10] ^ (*p).u.s[15] ^ (*p).u.s[20];
        c1 = (*p).u.s[1] ^ (*p).u.s[6] ^ (*p).u.s[11] ^ (*p).u.s[16] ^ (*p).u.s[21];
        c2 = (*p).u.s[2] ^ (*p).u.s[7] ^ (*p).u.s[12] ^ (*p).u.s[17] ^ (*p).u.s[22];
        c3 = (*p).u.s[3] ^ (*p).u.s[8] ^ (*p).u.s[13] ^ (*p).u.s[18] ^ (*p).u.s[23];
        c4 = (*p).u.s[4] ^ (*p).u.s[9] ^ (*p).u.s[14] ^ (*p).u.s[19] ^ (*p).u.s[24];
        d0 = c4 ^ (c1 << 1 | c1 >> 64 as i32 - 1);
        d1 = c0 ^ (c2 << 1 | c2 >> 64 as i32 - 1);
        d2 = c1 ^ (c3 << 1 | c3 >> 64 as i32 - 1);
        d3 = c2 ^ (c4 << 1 | c4 >> 64 as i32 - 1);
        d4 = c3 ^ (c0 << 1 | c0 >> 64 as i32 - 1);
        b0 = (*p).u.s[0] ^ d0;
        b1 = ((*p).u.s[6] ^ d1) << 44 as i32 | ((*p).u.s[6] ^ d1) >> 64 as i32 - 44 as i32;
        b2 = ((*p).u.s[12] ^ d2) << 43 | ((*p).u.s[12] ^ d2) >> 64 as i32 - 43;
        b3 = ((*p).u.s[18] ^ d3) << 21 | ((*p).u.s[18] ^ d3) >> 64 as i32 - 21;
        b4 = ((*p).u.s[24] ^ d4) << 14 as i32 | ((*p).u.s[24] ^ d4) >> 64 as i32 - 14 as i32;
        (*p).u.s[0] = b0 ^ !b1 & b2;
        (*p).u.s[0] ^= RC[i as usize];
        (*p).u.s[6] = b1 ^ !b2 & b3;
        (*p).u.s[12] = b2 ^ !b3 & b4;
        (*p).u.s[18] = b3 ^ !b4 & b0;
        (*p).u.s[24] = b4 ^ !b0 & b1;
        b2 = ((*p).u.s[10] ^ d0) << 3 | ((*p).u.s[10] ^ d0) >> 64 as i32 - 3;
        b3 = ((*p).u.s[16] ^ d1) << 45 | ((*p).u.s[16] ^ d1) >> 64 as i32 - 45;
        b4 = ((*p).u.s[22] ^ d2) << 61 | ((*p).u.s[22] ^ d2) >> 64 as i32 - 61;
        b0 = ((*p).u.s[3] ^ d3) << 28 | ((*p).u.s[3] ^ d3) >> 64 as i32 - 28;
        b1 = ((*p).u.s[9] ^ d4) << 20 | ((*p).u.s[9] ^ d4) >> 64 as i32 - 20;
        (*p).u.s[10] = b0 ^ !b1 & b2;
        (*p).u.s[16] = b1 ^ !b2 & b3;
        (*p).u.s[22] = b2 ^ !b3 & b4;
        (*p).u.s[3] = b3 ^ !b4 & b0;
        (*p).u.s[9] = b4 ^ !b0 & b1;
        b4 = ((*p).u.s[20] ^ d0) << 18 | ((*p).u.s[20] ^ d0) >> 64 as i32 - 18;
        b0 = ((*p).u.s[1] ^ d1) << 1 | ((*p).u.s[1] ^ d1) >> 64 as i32 - 1;
        b1 = ((*p).u.s[7] ^ d2) << 6 | ((*p).u.s[7] ^ d2) >> 64 as i32 - 6;
        b2 = ((*p).u.s[13] ^ d3) << 25 | ((*p).u.s[13] ^ d3) >> 64 as i32 - 25;
        b3 = ((*p).u.s[19] ^ d4) << 8 | ((*p).u.s[19] ^ d4) >> 64 as i32 - 8;
        (*p).u.s[20] = b0 ^ !b1 & b2;
        (*p).u.s[1] = b1 ^ !b2 & b3;
        (*p).u.s[7] = b2 ^ !b3 & b4;
        (*p).u.s[13] = b3 ^ !b4 & b0;
        (*p).u.s[19] = b4 ^ !b0 & b1;
        b1 = ((*p).u.s[5] ^ d0) << 36 | ((*p).u.s[5] ^ d0) >> 64 as i32 - 36;
        b2 = ((*p).u.s[11] ^ d1) << 10 | ((*p).u.s[11] ^ d1) >> 64 as i32 - 10;
        b3 = ((*p).u.s[17] ^ d2) << 15 | ((*p).u.s[17] ^ d2) >> 64 as i32 - 15;
        b4 = ((*p).u.s[23] ^ d3) << 56 | ((*p).u.s[23] ^ d3) >> 64 as i32 - 56;
        b0 = ((*p).u.s[4] ^ d4) << 27 as i32 | ((*p).u.s[4] ^ d4) >> 64 as i32 - 27 as i32;
        (*p).u.s[5] = b0 ^ !b1 & b2;
        (*p).u.s[11] = b1 ^ !b2 & b3;
        (*p).u.s[17] = b2 ^ !b3 & b4;
        (*p).u.s[23] = b3 ^ !b4 & b0;
        (*p).u.s[4] = b4 ^ !b0 & b1;
        b3 = ((*p).u.s[15] ^ d0) << 41 | ((*p).u.s[15] ^ d0) >> 64 as i32 - 41;
        b4 = ((*p).u.s[21] ^ d1) << 2 | ((*p).u.s[21] ^ d1) >> 64 as i32 - 2;
        b0 = ((*p).u.s[2] ^ d2) << 62 | ((*p).u.s[2] ^ d2) >> 64 as i32 - 62;
        b1 = ((*p).u.s[8] ^ d3) << 55 | ((*p).u.s[8] ^ d3) >> 64 as i32 - 55;
        b2 = ((*p).u.s[14] ^ d4) << 39 | ((*p).u.s[14] ^ d4) >> 64 as i32 - 39;
        (*p).u.s[15] = b0 ^ !b1 & b2;
        (*p).u.s[21] = b1 ^ !b2 & b3;
        (*p).u.s[2] = b2 ^ !b3 & b4;
        (*p).u.s[8] = b3 ^ !b4 & b0;
        (*p).u.s[14] = b4 ^ !b0 & b1;
        c0 = (*p).u.s[0] ^ (*p).u.s[10] ^ (*p).u.s[20] ^ (*p).u.s[5] ^ (*p).u.s[15];
        c1 = (*p).u.s[6] ^ (*p).u.s[16] ^ (*p).u.s[1] ^ (*p).u.s[11] ^ (*p).u.s[21];
        c2 = (*p).u.s[12] ^ (*p).u.s[22] ^ (*p).u.s[7] ^ (*p).u.s[17] ^ (*p).u.s[2];
        c3 = (*p).u.s[18] ^ (*p).u.s[3] ^ (*p).u.s[13] ^ (*p).u.s[23] ^ (*p).u.s[8];
        c4 = (*p).u.s[24] ^ (*p).u.s[9] ^ (*p).u.s[19] ^ (*p).u.s[4] ^ (*p).u.s[14];
        d0 = c4 ^ (c1 << 1 | c1 >> 64 as i32 - 1);
        d1 = c0 ^ (c2 << 1 | c2 >> 64 as i32 - 1);
        d2 = c1 ^ (c3 << 1 | c3 >> 64 as i32 - 1);
        d3 = c2 ^ (c4 << 1 | c4 >> 64 as i32 - 1);
        d4 = c3 ^ (c0 << 1 | c0 >> 64 as i32 - 1);
        b0 = (*p).u.s[0] ^ d0;
        b1 = ((*p).u.s[16] ^ d1) << 44 as i32 | ((*p).u.s[16] ^ d1) >> 64 as i32 - 44 as i32;
        b2 = ((*p).u.s[7] ^ d2) << 43 | ((*p).u.s[7] ^ d2) >> 64 as i32 - 43;
        b3 = ((*p).u.s[23] ^ d3) << 21 | ((*p).u.s[23] ^ d3) >> 64 as i32 - 21;
        b4 = ((*p).u.s[14] ^ d4) << 14 as i32 | ((*p).u.s[14] ^ d4) >> 64 as i32 - 14 as i32;
        (*p).u.s[0] = b0 ^ !b1 & b2;
        (*p).u.s[0] ^= RC[(i + 1) as usize];
        (*p).u.s[16] = b1 ^ !b2 & b3;
        (*p).u.s[7] = b2 ^ !b3 & b4;
        (*p).u.s[23] = b3 ^ !b4 & b0;
        (*p).u.s[14] = b4 ^ !b0 & b1;
        b2 = ((*p).u.s[20] ^ d0) << 3 | ((*p).u.s[20] ^ d0) >> 64 as i32 - 3;
        b3 = ((*p).u.s[11] ^ d1) << 45 | ((*p).u.s[11] ^ d1) >> 64 as i32 - 45;
        b4 = ((*p).u.s[2] ^ d2) << 61 | ((*p).u.s[2] ^ d2) >> 64 as i32 - 61;
        b0 = ((*p).u.s[18] ^ d3) << 28 | ((*p).u.s[18] ^ d3) >> 64 as i32 - 28;
        b1 = ((*p).u.s[9] ^ d4) << 20 | ((*p).u.s[9] ^ d4) >> 64 as i32 - 20;
        (*p).u.s[20] = b0 ^ !b1 & b2;
        (*p).u.s[11] = b1 ^ !b2 & b3;
        (*p).u.s[2] = b2 ^ !b3 & b4;
        (*p).u.s[18] = b3 ^ !b4 & b0;
        (*p).u.s[9] = b4 ^ !b0 & b1;
        b4 = ((*p).u.s[15] ^ d0) << 18 | ((*p).u.s[15] ^ d0) >> 64 as i32 - 18;
        b0 = ((*p).u.s[6] ^ d1) << 1 | ((*p).u.s[6] ^ d1) >> 64 as i32 - 1;
        b1 = ((*p).u.s[22] ^ d2) << 6 | ((*p).u.s[22] ^ d2) >> 64 as i32 - 6;
        b2 = ((*p).u.s[13] ^ d3) << 25 | ((*p).u.s[13] ^ d3) >> 64 as i32 - 25;
        b3 = ((*p).u.s[4] ^ d4) << 8 | ((*p).u.s[4] ^ d4) >> 64 as i32 - 8;
        (*p).u.s[15] = b0 ^ !b1 & b2;
        (*p).u.s[6] = b1 ^ !b2 & b3;
        (*p).u.s[22] = b2 ^ !b3 & b4;
        (*p).u.s[13] = b3 ^ !b4 & b0;
        (*p).u.s[4] = b4 ^ !b0 & b1;
        b1 = ((*p).u.s[10] ^ d0) << 36 | ((*p).u.s[10] ^ d0) >> 64 as i32 - 36;
        b2 = ((*p).u.s[1] ^ d1) << 10 | ((*p).u.s[1] ^ d1) >> 64 as i32 - 10;
        b3 = ((*p).u.s[17] ^ d2) << 15 | ((*p).u.s[17] ^ d2) >> 64 as i32 - 15;
        b4 = ((*p).u.s[8] ^ d3) << 56 | ((*p).u.s[8] ^ d3) >> 64 as i32 - 56;
        b0 = ((*p).u.s[24] ^ d4) << 27 as i32 | ((*p).u.s[24] ^ d4) >> 64 as i32 - 27 as i32;
        (*p).u.s[10] = b0 ^ !b1 & b2;
        (*p).u.s[1] = b1 ^ !b2 & b3;
        (*p).u.s[17] = b2 ^ !b3 & b4;
        (*p).u.s[8] = b3 ^ !b4 & b0;
        (*p).u.s[24] = b4 ^ !b0 & b1;
        b3 = ((*p).u.s[5] ^ d0) << 41 | ((*p).u.s[5] ^ d0) >> 64 as i32 - 41;
        b4 = ((*p).u.s[21] ^ d1) << 2 | ((*p).u.s[21] ^ d1) >> 64 as i32 - 2;
        b0 = ((*p).u.s[12] ^ d2) << 62 | ((*p).u.s[12] ^ d2) >> 64 as i32 - 62;
        b1 = ((*p).u.s[3] ^ d3) << 55 | ((*p).u.s[3] ^ d3) >> 64 as i32 - 55;
        b2 = ((*p).u.s[19] ^ d4) << 39 | ((*p).u.s[19] ^ d4) >> 64 as i32 - 39;
        (*p).u.s[5] = b0 ^ !b1 & b2;
        (*p).u.s[21] = b1 ^ !b2 & b3;
        (*p).u.s[12] = b2 ^ !b3 & b4;
        (*p).u.s[3] = b3 ^ !b4 & b0;
        (*p).u.s[19] = b4 ^ !b0 & b1;
        c0 = (*p).u.s[0] ^ (*p).u.s[20] ^ (*p).u.s[15] ^ (*p).u.s[10] ^ (*p).u.s[5];
        c1 = (*p).u.s[16] ^ (*p).u.s[11] ^ (*p).u.s[6] ^ (*p).u.s[1] ^ (*p).u.s[21];
        c2 = (*p).u.s[7] ^ (*p).u.s[2] ^ (*p).u.s[22] ^ (*p).u.s[17] ^ (*p).u.s[12];
        c3 = (*p).u.s[23] ^ (*p).u.s[18] ^ (*p).u.s[13] ^ (*p).u.s[8] ^ (*p).u.s[3];
        c4 = (*p).u.s[14] ^ (*p).u.s[9] ^ (*p).u.s[4] ^ (*p).u.s[24] ^ (*p).u.s[19];
        d0 = c4 ^ (c1 << 1 | c1 >> 64 as i32 - 1);
        d1 = c0 ^ (c2 << 1 | c2 >> 64 as i32 - 1);
        d2 = c1 ^ (c3 << 1 | c3 >> 64 as i32 - 1);
        d3 = c2 ^ (c4 << 1 | c4 >> 64 as i32 - 1);
        d4 = c3 ^ (c0 << 1 | c0 >> 64 as i32 - 1);
        b0 = (*p).u.s[0] ^ d0;
        b1 = ((*p).u.s[11] ^ d1) << 44 as i32 | ((*p).u.s[11] ^ d1) >> 64 as i32 - 44 as i32;
        b2 = ((*p).u.s[22] ^ d2) << 43 | ((*p).u.s[22] ^ d2) >> 64 as i32 - 43;
        b3 = ((*p).u.s[8] ^ d3) << 21 | ((*p).u.s[8] ^ d3) >> 64 as i32 - 21;
        b4 = ((*p).u.s[19] ^ d4) << 14 as i32 | ((*p).u.s[19] ^ d4) >> 64 as i32 - 14 as i32;
        (*p).u.s[0] = b0 ^ !b1 & b2;
        (*p).u.s[0] ^= RC[(i + 2) as usize];
        (*p).u.s[11] = b1 ^ !b2 & b3;
        (*p).u.s[22] = b2 ^ !b3 & b4;
        (*p).u.s[8] = b3 ^ !b4 & b0;
        (*p).u.s[19] = b4 ^ !b0 & b1;
        b2 = ((*p).u.s[15] ^ d0) << 3 | ((*p).u.s[15] ^ d0) >> 64 as i32 - 3;
        b3 = ((*p).u.s[1] ^ d1) << 45 | ((*p).u.s[1] ^ d1) >> 64 as i32 - 45;
        b4 = ((*p).u.s[12] ^ d2) << 61 | ((*p).u.s[12] ^ d2) >> 64 as i32 - 61;
        b0 = ((*p).u.s[23] ^ d3) << 28 | ((*p).u.s[23] ^ d3) >> 64 as i32 - 28;
        b1 = ((*p).u.s[9] ^ d4) << 20 | ((*p).u.s[9] ^ d4) >> 64 as i32 - 20;
        (*p).u.s[15] = b0 ^ !b1 & b2;
        (*p).u.s[1] = b1 ^ !b2 & b3;
        (*p).u.s[12] = b2 ^ !b3 & b4;
        (*p).u.s[23] = b3 ^ !b4 & b0;
        (*p).u.s[9] = b4 ^ !b0 & b1;
        b4 = ((*p).u.s[5] ^ d0) << 18 | ((*p).u.s[5] ^ d0) >> 64 as i32 - 18;
        b0 = ((*p).u.s[16] ^ d1) << 1 | ((*p).u.s[16] ^ d1) >> 64 as i32 - 1;
        b1 = ((*p).u.s[2] ^ d2) << 6 | ((*p).u.s[2] ^ d2) >> 64 as i32 - 6;
        b2 = ((*p).u.s[13] ^ d3) << 25 | ((*p).u.s[13] ^ d3) >> 64 as i32 - 25;
        b3 = ((*p).u.s[24] ^ d4) << 8 | ((*p).u.s[24] ^ d4) >> 64 as i32 - 8;
        (*p).u.s[5] = b0 ^ !b1 & b2;
        (*p).u.s[16] = b1 ^ !b2 & b3;
        (*p).u.s[2] = b2 ^ !b3 & b4;
        (*p).u.s[13] = b3 ^ !b4 & b0;
        (*p).u.s[24] = b4 ^ !b0 & b1;
        b1 = ((*p).u.s[20] ^ d0) << 36 | ((*p).u.s[20] ^ d0) >> 64 as i32 - 36;
        b2 = ((*p).u.s[6] ^ d1) << 10 | ((*p).u.s[6] ^ d1) >> 64 as i32 - 10;
        b3 = ((*p).u.s[17] ^ d2) << 15 | ((*p).u.s[17] ^ d2) >> 64 as i32 - 15;
        b4 = ((*p).u.s[3] ^ d3) << 56 | ((*p).u.s[3] ^ d3) >> 64 as i32 - 56;
        b0 = ((*p).u.s[14] ^ d4) << 27 as i32 | ((*p).u.s[14] ^ d4) >> 64 as i32 - 27 as i32;
        (*p).u.s[20] = b0 ^ !b1 & b2;
        (*p).u.s[6] = b1 ^ !b2 & b3;
        (*p).u.s[17] = b2 ^ !b3 & b4;
        (*p).u.s[3] = b3 ^ !b4 & b0;
        (*p).u.s[14] = b4 ^ !b0 & b1;
        b3 = ((*p).u.s[10] ^ d0) << 41 | ((*p).u.s[10] ^ d0) >> 64 as i32 - 41;
        b4 = ((*p).u.s[21] ^ d1) << 2 | ((*p).u.s[21] ^ d1) >> 64 as i32 - 2;
        b0 = ((*p).u.s[7] ^ d2) << 62 | ((*p).u.s[7] ^ d2) >> 64 as i32 - 62;
        b1 = ((*p).u.s[18] ^ d3) << 55 | ((*p).u.s[18] ^ d3) >> 64 as i32 - 55;
        b2 = ((*p).u.s[4] ^ d4) << 39 | ((*p).u.s[4] ^ d4) >> 64 as i32 - 39;
        (*p).u.s[10] = b0 ^ !b1 & b2;
        (*p).u.s[21] = b1 ^ !b2 & b3;
        (*p).u.s[7] = b2 ^ !b3 & b4;
        (*p).u.s[18] = b3 ^ !b4 & b0;
        (*p).u.s[4] = b4 ^ !b0 & b1;
        c0 = (*p).u.s[0] ^ (*p).u.s[15] ^ (*p).u.s[5] ^ (*p).u.s[20] ^ (*p).u.s[10];
        c1 = (*p).u.s[11] ^ (*p).u.s[1] ^ (*p).u.s[16] ^ (*p).u.s[6] ^ (*p).u.s[21];
        c2 = (*p).u.s[22] ^ (*p).u.s[12] ^ (*p).u.s[2] ^ (*p).u.s[17] ^ (*p).u.s[7];
        c3 = (*p).u.s[8] ^ (*p).u.s[23] ^ (*p).u.s[13] ^ (*p).u.s[3] ^ (*p).u.s[18];
        c4 = (*p).u.s[19] ^ (*p).u.s[9] ^ (*p).u.s[24] ^ (*p).u.s[14] ^ (*p).u.s[4];
        d0 = c4 ^ (c1 << 1 | c1 >> 64 as i32 - 1);
        d1 = c0 ^ (c2 << 1 | c2 >> 64 as i32 - 1);
        d2 = c1 ^ (c3 << 1 | c3 >> 64 as i32 - 1);
        d3 = c2 ^ (c4 << 1 | c4 >> 64 as i32 - 1);
        d4 = c3 ^ (c0 << 1 | c0 >> 64 as i32 - 1);
        b0 = (*p).u.s[0] ^ d0;
        b1 = ((*p).u.s[1] ^ d1) << 44 as i32 | ((*p).u.s[1] ^ d1) >> 64 as i32 - 44 as i32;
        b2 = ((*p).u.s[2] ^ d2) << 43 | ((*p).u.s[2] ^ d2) >> 64 as i32 - 43;
        b3 = ((*p).u.s[3] ^ d3) << 21 | ((*p).u.s[3] ^ d3) >> 64 as i32 - 21;
        b4 = ((*p).u.s[4] ^ d4) << 14 as i32 | ((*p).u.s[4] ^ d4) >> 64 as i32 - 14 as i32;
        (*p).u.s[0] = b0 ^ !b1 & b2;
        (*p).u.s[0] ^= RC[(i + 3) as usize];
        (*p).u.s[1] = b1 ^ !b2 & b3;
        (*p).u.s[2] = b2 ^ !b3 & b4;
        (*p).u.s[3] = b3 ^ !b4 & b0;
        (*p).u.s[4] = b4 ^ !b0 & b1;
        b2 = ((*p).u.s[5] ^ d0) << 3 | ((*p).u.s[5] ^ d0) >> 64 as i32 - 3;
        b3 = ((*p).u.s[6] ^ d1) << 45 | ((*p).u.s[6] ^ d1) >> 64 as i32 - 45;
        b4 = ((*p).u.s[7] ^ d2) << 61 | ((*p).u.s[7] ^ d2) >> 64 as i32 - 61;
        b0 = ((*p).u.s[8] ^ d3) << 28 | ((*p).u.s[8] ^ d3) >> 64 as i32 - 28;
        b1 = ((*p).u.s[9] ^ d4) << 20 | ((*p).u.s[9] ^ d4) >> 64 as i32 - 20;
        (*p).u.s[5] = b0 ^ !b1 & b2;
        (*p).u.s[6] = b1 ^ !b2 & b3;
        (*p).u.s[7] = b2 ^ !b3 & b4;
        (*p).u.s[8] = b3 ^ !b4 & b0;
        (*p).u.s[9] = b4 ^ !b0 & b1;
        b4 = ((*p).u.s[10] ^ d0) << 18 | ((*p).u.s[10] ^ d0) >> 64 as i32 - 18;
        b0 = ((*p).u.s[11] ^ d1) << 1 | ((*p).u.s[11] ^ d1) >> 64 as i32 - 1;
        b1 = ((*p).u.s[12] ^ d2) << 6 | ((*p).u.s[12] ^ d2) >> 64 as i32 - 6;
        b2 = ((*p).u.s[13] ^ d3) << 25 | ((*p).u.s[13] ^ d3) >> 64 as i32 - 25;
        b3 = ((*p).u.s[14] ^ d4) << 8 | ((*p).u.s[14] ^ d4) >> 64 as i32 - 8;
        (*p).u.s[10] = b0 ^ !b1 & b2;
        (*p).u.s[11] = b1 ^ !b2 & b3;
        (*p).u.s[12] = b2 ^ !b3 & b4;
        (*p).u.s[13] = b3 ^ !b4 & b0;
        (*p).u.s[14] = b4 ^ !b0 & b1;
        b1 = ((*p).u.s[15] ^ d0) << 36 | ((*p).u.s[15] ^ d0) >> 64 as i32 - 36;
        b2 = ((*p).u.s[16] ^ d1) << 10 | ((*p).u.s[16] ^ d1) >> 64 as i32 - 10;
        b3 = ((*p).u.s[17] ^ d2) << 15 | ((*p).u.s[17] ^ d2) >> 64 as i32 - 15;
        b4 = ((*p).u.s[18] ^ d3) << 56 | ((*p).u.s[18] ^ d3) >> 64 as i32 - 56;
        b0 = ((*p).u.s[19] ^ d4) << 27 as i32 | ((*p).u.s[19] ^ d4) >> 64 as i32 - 27 as i32;
        (*p).u.s[15] = b0 ^ !b1 & b2;
        (*p).u.s[16] = b1 ^ !b2 & b3;
        (*p).u.s[17] = b2 ^ !b3 & b4;
        (*p).u.s[18] = b3 ^ !b4 & b0;
        (*p).u.s[19] = b4 ^ !b0 & b1;
        b3 = ((*p).u.s[20] ^ d0) << 41 | ((*p).u.s[20] ^ d0) >> 64 as i32 - 41;
        b4 = ((*p).u.s[21] ^ d1) << 2 | ((*p).u.s[21] ^ d1) >> 64 as i32 - 2;
        b0 = ((*p).u.s[22] ^ d2) << 62 | ((*p).u.s[22] ^ d2) >> 64 as i32 - 62;
        b1 = ((*p).u.s[23] ^ d3) << 55 | ((*p).u.s[23] ^ d3) >> 64 as i32 - 55;
        b2 = ((*p).u.s[24] ^ d4) << 39 | ((*p).u.s[24] ^ d4) >> 64 as i32 - 39;
        (*p).u.s[20] = b0 ^ !b1 & b2;
        (*p).u.s[21] = b1 ^ !b2 & b3;
        (*p).u.s[22] = b2 ^ !b3 & b4;
        (*p).u.s[23] = b3 ^ !b4 & b0;
        (*p).u.s[24] = b4 ^ !b0 & b1;
    }
}
unsafe extern "C" fn SHA3Init(mut p: *mut SHA3Context, mut iSize: i32) {
    memset(p as *mut libc::c_void, 0, ::std::mem::size_of::<SHA3Context>() as u64);
    if iSize >= 128 && iSize <= 512 {
        (*p).nRate = ((1600 - (iSize + 31 & !(31)) * 2) / 8) as u32;
    } else {
        (*p).nRate = ((1600 - 2 * 256) / 8) as u32;
    };
}
unsafe extern "C" fn SHA3Update(p: *mut SHA3Context, aData: *const u8, nData: u32) {
    let mut i: u32 = 0;
    if aData.is_null() {
        return;
    }
    if ((*p).nLoaded).wrapping_rem(8) == 0
        && aData.offset_from(0 as *const u8) as libc::c_long & 7 as i32 as libc::c_long == 0 as libc::c_long
    {
        while i + 7 < nData {
            (*p).u.s[(((*p).nLoaded) / 8) as usize] ^= *(&*aData.offset(i as isize) as *const u8 as *mut u64);
            (*p).nLoaded += 8;
            if (*p).nLoaded >= (*p).nRate {
                KeccakF1600Step(p);
                (*p).nLoaded = 0;
            }
            i += 8;
        }
    }

    while i < nData {
        (*p).u.x[(*p).nLoaded as usize] = ((*p).u.x[(*p).nLoaded as usize] as i32 ^ *aData.offset(i as isize) as i32) as u8;
        (*p).nLoaded += 1;
        if (*p).nLoaded == (*p).nRate {
            KeccakF1600Step(p);
            (*p).nLoaded = 0;
        }
        i += 1;
    }
}

unsafe extern "C" fn SHA3Final(mut p: *mut SHA3Context) -> *mut u8 {
    if (*p).nLoaded == ((*p).nRate).wrapping_sub(1) {
        let c1: u8 = 0x86 as u8;
        SHA3Update(p, &c1, 1);
    } else {
        let c2: u8 = 0x6 as u8;
        let c3: u8 = 0x80 as u8;
        SHA3Update(p, &c2, 1);
        (*p).nLoaded = ((*p).nRate).wrapping_sub(1);
        SHA3Update(p, &c3, 1);
    }
    for i in 0..(*p).nRate as u32 {
        (*p).u.x[(i + (*p).nRate) as usize] = (*p).u.x[(i ^ (*p).ixMask) as usize];
    }
    return &mut *((*p).u.x).as_mut_ptr().offset((*p).nRate as isize) as *mut u8;
}
unsafe extern "C" fn sha3Func(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut cx: SHA3Context = SHA3Context {
        u: C2RustUnnamed_15 { s: [0; 25] },
        nRate: 0,
        nLoaded: 0,
        ixMask: 0,
    };
    let mut eType: i32 = sqlite3_value_type(*argv.offset(0));
    let mut nByte: i32 = sqlite3_value_bytes(*argv.offset(0));
    let mut iSize: i32 = 0;
    if argc == 1 {
        iSize = 256;
    } else {
        iSize = sqlite3_value_int(*argv.offset(1));
        if iSize != 224 && iSize != 256 && iSize != 384 as i32 && iSize != 512 {
            sqlite3_result_error(
                context,
                b"SHA3 size should be one of: 224 256 384 512\0" as *const u8 as *const i8,
                -1,
            );
            return;
        }
    }
    if eType == 5 {
        return;
    }
    SHA3Init(&mut cx, iSize);
    if eType == 4 {
        SHA3Update(&mut cx, sqlite3_value_blob(*argv.offset(0)) as *const u8, nByte as u32);
    } else {
        SHA3Update(&mut cx, sqlite3_value_text(*argv.offset(0)), nByte as u32);
    }
    sqlite3_result_blob(
        context,
        SHA3Final(&mut cx) as *const libc::c_void,
        iSize / 8,
        ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
    );
}
unsafe extern "C" fn hash_step_vformat(mut p: *mut SHA3Context, mut zFormat: *const i8, mut args: ...) {
    let mut ap: ::std::ffi::VaListImpl;
    let mut n: i32 = 0;
    let mut zBuf: [i8; 50] = [0; 50];
    ap = args.clone();
    sqlite3_vsnprintf(
        ::std::mem::size_of::<[i8; 50]>() as u64 as i32,
        zBuf.as_mut_ptr(),
        zFormat,
        ap.as_va_list(),
    );
    n = strlen(zBuf.as_mut_ptr()) as i32;
    SHA3Update(p, zBuf.as_mut_ptr() as *mut u8, n as u32);
}
unsafe extern "C" fn sha3QueryFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut db: *mut sqlite3 = sqlite3_context_db_handle(context);
    let mut zSql: *const i8 = sqlite3_value_text(*argv.offset(0)) as *const i8;
    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut nCol: i32 = 0;
    let mut i: i32 = 0;
    let mut rc: i32 = 0;
    let mut n: i32 = 0;
    let mut z: *const i8 = 0 as *const i8;
    let mut cx: SHA3Context = SHA3Context {
        u: C2RustUnnamed_15 { s: [0; 25] },
        nRate: 0,
        nLoaded: 0,
        ixMask: 0,
    };
    let mut iSize: i32 = 0;
    if argc == 1 {
        iSize = 256;
    } else {
        iSize = sqlite3_value_int(*argv.offset(1));
        if iSize != 224 && iSize != 256 && iSize != 384 as i32 && iSize != 512 {
            sqlite3_result_error(
                context,
                b"SHA3 size should be one of: 224 256 384 512\0" as *const u8 as *const i8,
                -1,
            );
            return;
        }
    }
    if zSql.is_null() {
        return;
    }
    SHA3Init(&mut cx, iSize);
    while *zSql.offset(0) != 0 {
        rc = sqlite3_prepare_v2(db, zSql, -1, &mut pStmt, &mut zSql);
        if rc != 0 {
            let mut zMsg: *mut i8 = sqlite3_mprintf(
                b"error SQL statement [%s]: %s\0" as *const u8 as *const i8,
                zSql,
                sqlite3_errmsg(db),
            );
            sqlite3_finalize(pStmt);
            sqlite3_result_error(context, zMsg, -1);
            sqlite3_free(zMsg as *mut libc::c_void);
            return;
        }
        if sqlite3_stmt_readonly(pStmt) == 0 {
            let mut zMsg_0: *mut i8 = sqlite3_mprintf(b"non-query: [%s]\0" as *const u8 as *const i8, sqlite3_sql(pStmt));
            sqlite3_finalize(pStmt);
            sqlite3_result_error(context, zMsg_0, -1);
            sqlite3_free(zMsg_0 as *mut libc::c_void);
            return;
        }
        nCol = sqlite3_column_count(pStmt);
        z = sqlite3_sql(pStmt);
        if !z.is_null() {
            n = strlen(z) as i32;
            hash_step_vformat(&mut cx as *mut SHA3Context, b"S%d:\0" as *const u8 as *const i8, n);
            SHA3Update(&mut cx, z as *mut u8, n as u32);
        }
        while 100 == sqlite3_step(pStmt) {
            SHA3Update(&mut cx, b"R\0" as *const u8 as *const i8 as *const u8, 1);
            for i in 0..nCol {
                match sqlite3_column_type(pStmt, i) {
                    5 => {
                        SHA3Update(&mut cx, b"N\0" as *const u8 as *const i8 as *const u8, 1);
                    }
                    1 => {
                        let mut u: u64 = 0;
                        let mut j: i32 = 0;
                        let mut x: [u8; 9] = [0; 9];
                        let mut v: i64 = sqlite3_column_int64(pStmt, i);
                        memcpy(
                            &mut u as *mut u64 as *mut libc::c_void,
                            &mut v as *mut i64 as *const libc::c_void,
                            8 as u64,
                        );
                        j = 8;
                        while j >= 1 {
                            x[j as usize] = (u & 0xff as i32 as u64) as u8;
                            u >>= 8;
                            j -= 1;
                        }
                        x[0] = 'I' as i32 as u8;
                        SHA3Update(&mut cx, x.as_mut_ptr(), 9 as u32);
                    }
                    2 => {
                        let mut u_0: u64 = 0;
                        let mut j_0: i32 = 0;
                        let mut x_0: [u8; 9] = [0; 9];
                        let mut r: f64 = sqlite3_column_double(pStmt, i);
                        memcpy(
                            &mut u_0 as *mut u64 as *mut libc::c_void,
                            &mut r as *mut f64 as *const libc::c_void,
                            8 as u64,
                        );
                        j_0 = 8;
                        while j_0 >= 1 {
                            x_0[j_0 as usize] = (u_0 & 0xff as i32 as u64) as u8;
                            u_0 >>= 8;
                            j_0 -= 1;
                        }
                        x_0[0] = 'F' as i32 as u8;
                        SHA3Update(&mut cx, x_0.as_mut_ptr(), 9 as u32);
                    }
                    3 => {
                        let mut n2: i32 = sqlite3_column_bytes(pStmt, i);
                        let mut z2: *const u8 = sqlite3_column_text(pStmt, i);
                        hash_step_vformat(&mut cx as *mut SHA3Context, b"T%d:\0" as *const u8 as *const i8, n2);
                        SHA3Update(&mut cx, z2, n2 as u32);
                    }
                    4 => {
                        let mut n2_0: i32 = sqlite3_column_bytes(pStmt, i);
                        let mut z2_0: *const u8 = sqlite3_column_blob(pStmt, i) as *const u8;
                        hash_step_vformat(&mut cx as *mut SHA3Context, b"B%d:\0" as *const u8 as *const i8, n2_0);
                        SHA3Update(&mut cx, z2_0, n2_0 as u32);
                    }
                    _ => {}
                }
            }
        }
        sqlite3_finalize(pStmt);
    }
    sqlite3_result_blob(
        context,
        SHA3Final(&mut cx) as *const libc::c_void,
        iSize / 8,
        ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_shathree_init(
    mut db: *mut sqlite3,
    mut pzErrMsg: *mut *mut i8,
    mut pApi: *const sqlite3_api_routines,
) -> i32 {
    let mut rc: i32 = 0;
    rc = sqlite3_create_function(
        db,
        b"sha3\0" as *const u8 as *const i8,
        1,
        1 | 0x200000 | 0x800,
        std::ptr::null_mut(),
        Some(sha3Func),
        None,
        None,
    );
    if rc == 0 {
        rc = sqlite3_create_function(
            db,
            b"sha3\0" as *const u8 as *const i8,
            2,
            1 | 0x200000 | 0x800,
            std::ptr::null_mut(),
            Some(sha3Func),
            None,
            None,
        );
    }
    if rc == 0 {
        rc = sqlite3_create_function(
            db,
            b"sha3_query\0" as *const u8 as *const i8,
            1,
            1 | 0x80000,
            std::ptr::null_mut(),
            Some(sha3QueryFunc),
            None,
            None,
        );
    }
    if rc == 0 {
        rc = sqlite3_create_function(
            db,
            b"sha3_query\0" as *const u8 as *const i8,
            2,
            1 | 0x80000,
            std::ptr::null_mut(),
            Some(sha3QueryFunc),
            None,
            None,
        );
    }
    return rc;
}
unsafe extern "C" fn readFileContents(mut ctx: *mut sqlite3_context, mut zName: *const i8) {
    let mut in_0: *mut FILE = 0 as *mut FILE;
    let mut nIn: i64 = 0;
    let mut pBuf: *mut libc::c_void = std::ptr::null_mut();
    let mut db: *mut sqlite3 = 0 as *mut sqlite3;
    let mut mxBlob: i32 = 0;
    in_0 = fopen(zName, b"rb\0" as *const u8 as *const i8);
    if in_0.is_null() {
        return;
    }
    fseek(in_0, 0 as libc::c_long, 2);
    nIn = ftell(in_0) as i64;
    rewind(in_0);
    db = sqlite3_context_db_handle(ctx);
    mxBlob = sqlite3_limit(db, 0, -1);
    if nIn > mxBlob as i64 {
        sqlite3_result_error_code(ctx, 18);
        fclose(in_0);
        return;
    }
    pBuf = sqlite3_malloc64((if nIn != 0 { nIn } else { 1 as i64 }) as u64);
    if pBuf.is_null() {
        sqlite3_result_error_nomem(ctx);
        fclose(in_0);
        return;
    }
    if nIn == fread(pBuf, 1 as u64, nIn as size_t, in_0) as i64 {
        sqlite3_result_blob64(ctx, pBuf, nIn as u64, Some(sqlite3_free));
    } else {
        sqlite3_result_error_code(ctx, 10);
        sqlite3_free(pBuf);
    }
    fclose(in_0);
}
unsafe extern "C" fn readfileFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut zName: *const i8 = 0 as *const i8;
    zName = sqlite3_value_text(*argv.offset(0)) as *const i8;
    if zName.is_null() {
        return;
    }
    readFileContents(context, zName);
}
unsafe extern "C" fn ctxErrorMsg(mut ctx: *mut sqlite3_context, mut zFmt: *const i8, mut args: ...) {
    let mut zMsg: *mut i8 = std::ptr::null_mut();
    let mut ap: ::std::ffi::VaListImpl;
    ap = args.clone();
    zMsg = sqlite3_vmprintf(zFmt, ap.as_va_list());
    sqlite3_result_error(ctx, zMsg, -1);
    sqlite3_free(zMsg as *mut libc::c_void);
}

unsafe extern "C" fn fileLinkStat(mut zPath: *const i8, mut pStatBuf: *mut stat) -> i32 {
    return lstat(zPath, pStatBuf);
}

/*
** Argument zFile is the name of a file that will be created and/or written
** by SQL function writefile(). This function ensures that the directory
** zFile will be written to exists, creating it if required. The permissions
** for any path components created by this function are set in accordance
** with the current umask.
*/
fn makeDirectory(zFile: &str) -> bool {
    let path = std::path::Path::new(zFile);
    match std::fs::create_dir_all(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

unsafe extern "C" fn writeFile(
    mut pCtx: *mut sqlite3_context,
    mut zFile: *const i8,
    mut pData: *mut sqlite3_value,
    mut mode: mode_t,
    mut mtime: i64,
) -> i32 {
    if zFile.is_null() {
        return 1;
    }
    if mode & 0o170000 == 0o120000 {
        let mut zTo: *const i8 = sqlite3_value_text(pData) as *const i8;
        if zTo.is_null() || symlink(zTo, zFile) < 0 {
            return 1;
        }
    } else if mode & 0o170000 == 0o40000 {
        let zFile = cstr_to_string!(zFile);
        let path = std::path::Path::new(&*zFile);
        let result = std::fs::create_dir(path);

        use std::os::unix::fs::PermissionsExt;
        /* The mkdir() call to create the directory failed. This might not
         ** be an error though - if there is already a directory at the same
         ** path and either the permissions already match or can be changed
         ** to do so using chmod(), it is not an error.  */
        match result {
            Ok(_) => {}
            Err(e) => match std::fs::metadata(path) {
                Ok(attr) => {
                    if attr.is_dir() && (attr.mode() & 0o777) != (mode & 0o777) {
                        let mut permissions = attr.permissions();
                        permissions.set_mode(mode & 0o777);
                        let r = std::fs::set_permissions(path, permissions);
                        if r.is_ok() {
                            return 1;
                        }
                    }
                }
                Err(_) => {}
            },
        }
    } else {
        let mut nWrite: i64 = 0;
        let mut z: *const i8 = 0 as *const i8;
        let mut rc: i32 = 0;
        let mut out: *mut FILE = fopen(zFile, b"wb\0" as *const u8 as *const i8);
        if out.is_null() {
            return 1;
        }
        z = sqlite3_value_blob(pData) as *const i8;
        if !z.is_null() {
            let mut n: i64 = fwrite(z as *const libc::c_void, 1 as u64, sqlite3_value_bytes(pData) as u64, out) as i64;
            nWrite = sqlite3_value_bytes(pData) as i64;
            if nWrite != n {
                rc = 1;
            }
        }
        fclose(out);
        if rc == 0 && mode != 0 && chmod(zFile, mode & 0o777 as i32 as u32) != 0 {
            rc = 1;
        }
        if rc != 0 {
            return 2;
        }
        sqlite3_result_int64(pCtx, nWrite);
    }
    if mtime >= 0 {
        let mut times: [timeval; 2] = [timeval { tv_sec: 0, tv_usec: 0 }; 2];
        times[1].tv_usec = 0 as __suseconds_t;
        times[0].tv_usec = times[1].tv_usec;
        times[0].tv_sec = time(0 as *mut time_t);
        times[1].tv_sec = mtime as __time_t;
        if utimes(zFile, times.as_mut_ptr() as *const timeval) != 0 {
            return 1;
        }
    }
    return 0;
}
unsafe extern "C" fn writefileFunc(mut context: *mut sqlite3_context, argc: i32, argv: *mut *mut sqlite3_value) {
    let mut zFile: *const i8 = 0 as *const i8;
    let mut mode: mode_t = 0 as mode_t;
    let mut res: i32 = 0;
    let mut mtime: i64 = -1;
    if argc < 2 || argc > 4 {
        sqlite3_result_error(
            context,
            b"wrong number of arguments to function writefile()\0" as *const u8 as *const i8,
            -1,
        );
        return;
    }
    zFile = sqlite3_value_text(*argv.offset(0)) as *const i8;
    if zFile.is_null() {
        return;
    }
    if argc >= 3 {
        mode = sqlite3_value_int(*argv.offset(2 as isize)) as mode_t;
    }
    if argc == 4 {
        mtime = sqlite3_value_int64(*argv.offset(3 as isize));
    }
    res = writeFile(context, zFile, *argv.offset(1), mode, mtime);
    if res == 1 && *__errno_location() == 2 {
        if makeDirectory(&cstr_to_string!(zFile)) {
            res = writeFile(context, zFile, *argv.offset(1), mode, mtime);
        }
    }
    if argc > 2 && res != 0 {
        if mode & 0o170000 == 0o120000 {
            ctxErrorMsg(context, b"failed to create symlink: %s\0" as *const u8 as *const i8, zFile);
        } else if mode & 0o170000 == 0o40000 {
            ctxErrorMsg(context, b"failed to create directory: %s\0" as *const u8 as *const i8, zFile);
        } else {
            ctxErrorMsg(context, b"failed to write file: %s\0" as *const u8 as *const i8, zFile);
        }
    }
}
unsafe extern "C" fn lsModeFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut iMode: i32 = sqlite3_value_int(*argv.offset(0));
    let mut z: [i8; 16] = [0; 16];
    if iMode & 0o170000 == 0o120000 {
        z[0] = 'l' as i32 as i8;
    } else if iMode & 0o170000 == 0o100000 {
        z[0] = '-' as i32 as i8;
    } else if iMode & 0o170000 == 0o40000 {
        z[0] = 'd' as i32 as i8;
    } else {
        z[0] = '?' as i32 as i8;
    }
    for i in 0..3 {
        let mut m: i32 = iMode >> (2 - i) * 3;
        let mut a: *mut i8 = &mut *z.as_mut_ptr().offset((1 + i * 3) as isize) as *mut i8;
        *a.offset(0) = (if m & 0x4 as i32 != 0 { 'r' as i32 } else { '-' as i32 }) as i8;
        *a.offset(1) = (if m & 0x2 != 0 { 'w' as i32 } else { '-' as i32 }) as i8;
        *a.offset(2) = (if m & 0x1 != 0 { 'x' as i32 } else { '-' as i32 }) as i8;
    }
    z[10] = '\0' as i32 as i8;
    sqlite3_result_text(
        context,
        z.as_mut_ptr(),
        -1,
        ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
    );
}
unsafe extern "C" fn fsdirConnect(
    mut db: *mut sqlite3,
    mut pAux: *mut libc::c_void,
    mut argc: i32,
    mut argv: *const *const i8,
    mut ppVtab: *mut *mut sqlite3_vtab,
    mut pzErr: *mut *mut i8,
) -> i32 {
    let mut pNew: *mut fsdir_tab = 0 as *mut fsdir_tab;
    let mut rc: i32 = 0;
    rc = sqlite3_declare_vtab(
        db,
        b"CREATE TABLE x(name,mode,mtime,data,path HIDDEN,dir HIDDEN)\0" as *const u8 as *const i8,
    );
    if rc == 0 {
        pNew = sqlite3_malloc(::std::mem::size_of::<fsdir_tab>() as u64 as i32) as *mut fsdir_tab;
        if pNew.is_null() {
            return 7 as i32;
        }
        memset(pNew as *mut libc::c_void, 0, ::std::mem::size_of::<fsdir_tab>() as u64);
        sqlite3_vtab_config(db, 3);
    }
    *ppVtab = pNew as *mut sqlite3_vtab;
    return rc;
}
unsafe extern "C" fn fsdirDisconnect(mut pVtab: *mut sqlite3_vtab) -> i32 {
    sqlite3_free(pVtab as *mut libc::c_void);
    return 0;
}
unsafe extern "C" fn fsdirOpen(mut p: *mut sqlite3_vtab, mut ppCursor: *mut *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut fsdir_cursor = 0 as *mut fsdir_cursor;
    pCur = sqlite3_malloc(::std::mem::size_of::<fsdir_cursor>() as u64 as i32) as *mut fsdir_cursor;
    if pCur.is_null() {
        return 7 as i32;
    }
    memset(pCur as *mut libc::c_void, 0, ::std::mem::size_of::<fsdir_cursor>() as u64);
    (*pCur).iLvl = -1;
    *ppCursor = &mut (*pCur).base;
    return 0;
}
unsafe extern "C" fn fsdirResetCursor(pCur: *mut fsdir_cursor) {
    for i in 0..=(*pCur).iLvl {
        let mut pLvl: *mut FsdirLevel = &mut *((*pCur).aLvl).offset(i as isize) as *mut FsdirLevel;
        if !((*pLvl).pDir).is_null() {
            closedir((*pLvl).pDir);
        }
        sqlite3_free((*pLvl).zDir as *mut libc::c_void);
    }
    sqlite3_free((*pCur).zPath as *mut libc::c_void);
    sqlite3_free((*pCur).aLvl as *mut libc::c_void);
    (*pCur).aLvl = 0 as *mut FsdirLevel;
    (*pCur).zPath = std::ptr::null_mut();
    (*pCur).zBase = 0 as *const i8;
    (*pCur).nBase = 0;
    (*pCur).nLvl = 0;
    (*pCur).iLvl = -1;
    (*pCur).iRowid = 1 as i64;
}
unsafe extern "C" fn fsdirClose(cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut fsdir_cursor = cur as *mut fsdir_cursor;
    fsdirResetCursor(pCur);
    sqlite3_free(pCur as *mut libc::c_void);
    return 0;
}
unsafe extern "C" fn fsdirSetErrmsg(pCur: *mut fsdir_cursor, mut zFmt: *const i8, mut args: ...) {
    let mut ap: ::std::ffi::VaListImpl;
    ap = args.clone();
    (*(*pCur).base.pVtab).zErrMsg = sqlite3_vmprintf(zFmt, ap.as_va_list());
}

unsafe extern "C" fn fsdirNext(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut fsdir_cursor = cur as *mut fsdir_cursor;
    let mut m: mode_t = (*pCur).sStat.st_mode;
    (*pCur).iRowid += 1;
    if m & 0o170000 == 0o40000 {
        let mut iNew: i32 = (*pCur).iLvl + 1;
        let mut pLvl: *mut FsdirLevel = 0 as *mut FsdirLevel;
        if iNew >= (*pCur).nLvl {
            let mut nNew: i32 = iNew + 1;
            let mut nByte: i64 = (nNew as u64).wrapping_mul(::std::mem::size_of::<FsdirLevel>() as u64) as i64;
            let mut aNew: *mut FsdirLevel = sqlite3_realloc64((*pCur).aLvl as *mut libc::c_void, nByte as u64) as *mut FsdirLevel;
            if aNew.is_null() {
                return 7 as i32;
            }
            memset(
                &mut *aNew.offset((*pCur).nLvl as isize) as *mut FsdirLevel as *mut libc::c_void,
                0,
                (::std::mem::size_of::<FsdirLevel>() as u64).wrapping_mul((nNew - (*pCur).nLvl) as u64),
            );
            (*pCur).aLvl = aNew;
            (*pCur).nLvl = nNew;
        }
        (*pCur).iLvl = iNew;
        pLvl = &mut *((*pCur).aLvl).offset(iNew as isize) as *mut FsdirLevel;
        (*pLvl).zDir = (*pCur).zPath;
        (*pCur).zPath = std::ptr::null_mut();
        (*pLvl).pDir = opendir((*pLvl).zDir);
        if ((*pLvl).pDir).is_null() {
            fsdirSetErrmsg(pCur, b"cannot read directory: %s\0" as *const u8 as *const i8, (*pCur).zPath);
            return 1;
        }
    }
    while (*pCur).iLvl >= 0 {
        let mut pLvl_0: *mut FsdirLevel = &mut *((*pCur).aLvl).offset((*pCur).iLvl as isize) as *mut FsdirLevel;
        let mut pEntry: *mut dirent = readdir((*pLvl_0).pDir);
        if !pEntry.is_null() {
            if (*pEntry).d_name[0] as i32 == '.' as i32 {
                if (*pEntry).d_name[1] as i32 == '.' as i32 && (*pEntry).d_name[2] as i32 == '\0' as i32 {
                    continue;
                }
                if (*pEntry).d_name[1] as i32 == '\0' as i32 {
                    continue;
                }
            }
            sqlite3_free((*pCur).zPath as *mut libc::c_void);
            (*pCur).zPath = sqlite3_mprintf(
                b"%s/%s\0" as *const u8 as *const i8,
                (*pLvl_0).zDir,
                ((*pEntry).d_name).as_mut_ptr(),
            );
            if ((*pCur).zPath).is_null() {
                return 7 as i32;
            }
            if fileLinkStat((*pCur).zPath, &mut (*pCur).sStat) != 0 {
                fsdirSetErrmsg(pCur, b"cannot stat file: %s\0" as *const u8 as *const i8, (*pCur).zPath);
                return 1;
            }
            return 0;
        } else {
            closedir((*pLvl_0).pDir);
            sqlite3_free((*pLvl_0).zDir as *mut libc::c_void);
            (*pLvl_0).pDir = 0 as *mut DIR;
            (*pLvl_0).zDir = std::ptr::null_mut();
            (*pCur).iLvl -= 1;
        }
    }
    sqlite3_free((*pCur).zPath as *mut libc::c_void);
    (*pCur).zPath = std::ptr::null_mut();
    return 0;
}
unsafe extern "C" fn fsdirColumn(mut cur: *mut sqlite3_vtab_cursor, mut ctx: *mut sqlite3_context, mut i: i32) -> i32 {
    let mut pCur: *mut fsdir_cursor = cur as *mut fsdir_cursor;
    match i {
        0 => {
            sqlite3_result_text(
                ctx,
                &mut *((*pCur).zPath).offset((*pCur).nBase as isize),
                -1,
                ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
            );
        }
        1 => {
            sqlite3_result_int64(ctx, (*pCur).sStat.st_mode as i64);
        }
        2 => {
            sqlite3_result_int64(ctx, (*pCur).sStat.st_mtim.tv_sec as i64);
        }
        3 => {
            let mut m: mode_t = (*pCur).sStat.st_mode;
            if m & 0o170000 == 0o40000 {
                sqlite3_result_null(ctx);
            } else if m & 0o170000 == 0o120000 {
                let mut aStatic: [i8; 64] = [0; 64];
                let mut aBuf: *mut i8 = aStatic.as_mut_ptr();
                let mut nBuf: i64 = 64 as i32 as i64;
                let mut n: i32 = 0;
                loop {
                    n = readlink((*pCur).zPath, aBuf, nBuf as size_t) as i32;
                    if (n as i64) < nBuf {
                        break;
                    }
                    if aBuf != aStatic.as_mut_ptr() {
                        sqlite3_free(aBuf as *mut libc::c_void);
                    }
                    nBuf = nBuf * 2 as i64;
                    aBuf = sqlite3_malloc64(nBuf as u64) as *mut i8;
                    if aBuf.is_null() {
                        sqlite3_result_error_nomem(ctx);
                        return 7 as i32;
                    }
                }
                sqlite3_result_text(
                    ctx,
                    aBuf,
                    n,
                    ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
                );
                if aBuf != aStatic.as_mut_ptr() {
                    sqlite3_free(aBuf as *mut libc::c_void);
                }
            } else {
                readFileContents(ctx, (*pCur).zPath);
            }
        }
        4 | _ => {}
    }
    return 0;
}
unsafe extern "C" fn fsdirRowid(cur: *mut sqlite3_vtab_cursor, mut pRowid: *mut i64) -> i32 {
    let pCur: *mut fsdir_cursor = cur as *mut fsdir_cursor;
    *pRowid = (*pCur).iRowid;
    return 0;
}
unsafe extern "C" fn fsdirEof(cur: *mut sqlite3_vtab_cursor) -> i32 {
    let pCur: *mut fsdir_cursor = cur as *mut fsdir_cursor;
    return ((*pCur).zPath == std::ptr::null_mut()) as i32;
}
unsafe extern "C" fn fsdirFilter(
    mut cur: *mut sqlite3_vtab_cursor,
    mut idxNum: i32,
    mut idxStr: *const i8,
    mut argc: i32,
    mut argv: *mut *mut sqlite3_value,
) -> i32 {
    let mut zDir: *const i8 = 0 as *const i8;
    let mut pCur: *mut fsdir_cursor = cur as *mut fsdir_cursor;
    fsdirResetCursor(pCur);
    if idxNum == 0 {
        fsdirSetErrmsg(pCur, b"table function fsdir requires an argument\0" as *const u8 as *const i8);
        return 1;
    }
    assert!(argc == idxNum && (argc == 1 || argc == 2));
    zDir = sqlite3_value_text(*argv.offset(0)) as *const i8;
    if zDir.is_null() {
        fsdirSetErrmsg(
            pCur,
            b"table function fsdir requires a non-NULL argument\0" as *const u8 as *const i8,
        );
        return 1;
    }
    if argc == 2 {
        (*pCur).zBase = sqlite3_value_text(*argv.offset(1)) as *const i8;
    }
    if !((*pCur).zBase).is_null() {
        (*pCur).nBase = strlen((*pCur).zBase) as i32 + 1;
        (*pCur).zPath = sqlite3_mprintf(b"%s/%s\0" as *const u8 as *const i8, (*pCur).zBase, zDir);
    } else {
        (*pCur).zPath = sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, zDir);
    }
    if ((*pCur).zPath).is_null() {
        return 7 as i32;
    }
    if fileLinkStat((*pCur).zPath, &mut (*pCur).sStat) != 0 {
        fsdirSetErrmsg(pCur, b"cannot stat file: %s\0" as *const u8 as *const i8, (*pCur).zPath);
        return 1;
    }
    return 0;
}
unsafe extern "C" fn fsdirBestIndex(mut tab: *mut sqlite3_vtab, mut pIdxInfo: *mut sqlite3_index_info) -> i32 {
    let mut idxPath: i32 = -1;
    let mut idxDir: i32 = -1;
    let mut seenPath: bool = false;
    let mut seenDir: bool = false;
    let mut pConstraint: *const sqlite3_index_constraint = (*pIdxInfo).aConstraint;

    for i in 0..(*pIdxInfo).nConstraint {
        if !((*pConstraint).op != 2) {
            match (*pConstraint).iColumn {
                4 => {
                    if (*pConstraint).usable != 0 {
                        idxPath = i;
                        seenPath = false;
                    } else if idxPath < 0 {
                        seenPath = true;
                    }
                }
                5 => {
                    if (*pConstraint).usable != 0 {
                        idxDir = i;
                        seenDir = false;
                    } else if idxDir < 0 {
                        seenDir = true;
                    }
                }
                _ => {}
            }
        }
        pConstraint = pConstraint.offset(1);
    }

    if seenPath || seenDir {
        return 19;
    }
    if idxPath < 0 {
        (*pIdxInfo).idxNum = 0;
        (*pIdxInfo).estimatedRows = 0x7fffffff as i32 as i64;
    } else {
        (*((*pIdxInfo).aConstraintUsage).offset(idxPath as isize)).omit = 1;
        (*((*pIdxInfo).aConstraintUsage).offset(idxPath as isize)).argvIndex = 1;
        if idxDir >= 0 {
            (*((*pIdxInfo).aConstraintUsage).offset(idxDir as isize)).omit = 1;
            (*((*pIdxInfo).aConstraintUsage).offset(idxDir as isize)).argvIndex = 2;
            (*pIdxInfo).idxNum = 2;
            (*pIdxInfo).estimatedCost = 10.0f64;
        } else {
            (*pIdxInfo).idxNum = 1;
            (*pIdxInfo).estimatedCost = 100.0f64;
        }
    }
    return 0;
}
unsafe extern "C" fn fsdirRegister(mut db: *mut sqlite3) -> i32 {
    static mut fsdirModule: sqlite3_module = unsafe {
        {
            let mut init = sqlite3_module {
                iVersion: 0,
                xCreate: None,
                xConnect: Some(fsdirConnect),
                xBestIndex: Some(fsdirBestIndex),
                xDisconnect: Some(fsdirDisconnect),
                xDestroy: None,
                xOpen: Some(fsdirOpen),
                xClose: Some(fsdirClose),
                xFilter: Some(fsdirFilter),
                xNext: Some(fsdirNext),
                xEof: Some(fsdirEof),
                xColumn: Some(fsdirColumn),
                xRowid: Some(fsdirRowid),
                xUpdate: None,
                xBegin: None,
                xSync: None,
                xCommit: None,
                xRollback: None,
                xFindFunction: None,
                xRename: None,
                xSavepoint: None,
                xRelease: None,
                xRollbackTo: None,
                xShadowName: None,
            };
            init
        }
    };
    let mut rc: i32 = sqlite3_create_module(db, b"fsdir\0" as *const u8 as *const i8, &mut fsdirModule, std::ptr::null_mut());
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_fileio_init(
    mut db: *mut sqlite3,
    mut pzErrMsg: *mut *mut i8,
    mut pApi: *const sqlite3_api_routines,
) -> i32 {
    let mut rc: i32 = 0;
    rc = sqlite3_create_function(
        db,
        b"readfile\0" as *const u8 as *const i8,
        1,
        1 | 0x80000,
        std::ptr::null_mut(),
        Some(readfileFunc),
        None,
        None,
    );
    if rc == 0 {
        rc = sqlite3_create_function(
            db,
            b"writefile\0" as *const u8 as *const i8,
            -1,
            1 | 0x80000,
            std::ptr::null_mut(),
            Some(writefileFunc),
            None,
            None,
        );
    }
    if rc == 0 {
        rc = sqlite3_create_function(
            db,
            b"lsmode\0" as *const u8 as *const i8,
            1,
            1,
            std::ptr::null_mut(),
            Some(lsModeFunc),
            None,
            None,
        );
    }
    if rc == 0 {
        rc = fsdirRegister(db);
    }
    return rc;
}
unsafe extern "C" fn completionConnect(
    mut db: *mut sqlite3,
    mut pAux: *mut libc::c_void,
    mut argc: i32,
    mut argv: *const *const i8,
    mut ppVtab: *mut *mut sqlite3_vtab,
    mut pzErr: *mut *mut i8,
) -> i32 {
    let mut pNew: *mut completion_vtab = 0 as *mut completion_vtab;
    let mut rc: i32 = 0;
    sqlite3_vtab_config(db, 2);
    rc = sqlite3_declare_vtab(
        db,
        b"CREATE TABLE x(  candidate TEXT,  prefix TEXT HIDDEN,  wholeline TEXT HIDDEN,  phase INT HIDDEN)\0" as *const u8 as *const i8,
    );
    if rc == 0 {
        pNew = sqlite3_malloc(::std::mem::size_of::<completion_vtab>() as u64 as i32) as *mut completion_vtab;
        *ppVtab = pNew as *mut sqlite3_vtab;
        if pNew.is_null() {
            return 7 as i32;
        }
        memset(pNew as *mut libc::c_void, 0, ::std::mem::size_of::<completion_vtab>() as u64);
        (*pNew).db = db;
    }
    return rc;
}
unsafe extern "C" fn completionDisconnect(mut pVtab: *mut sqlite3_vtab) -> i32 {
    sqlite3_free(pVtab as *mut libc::c_void);
    return 0;
}
unsafe extern "C" fn completionOpen(mut p: *mut sqlite3_vtab, mut ppCursor: *mut *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut completion_cursor = 0 as *mut completion_cursor;
    pCur = sqlite3_malloc(::std::mem::size_of::<completion_cursor>() as u64 as i32) as *mut completion_cursor;
    if pCur.is_null() {
        return 7 as i32;
    }
    memset(pCur as *mut libc::c_void, 0, ::std::mem::size_of::<completion_cursor>() as u64);
    (*pCur).db = (*(p as *mut completion_vtab)).db;
    *ppCursor = &mut (*pCur).base;
    return 0;
}

unsafe extern "C" fn completionCursorReset(mut pCur: *mut completion_cursor) {
    sqlite3_free((*pCur).zPrefix as *mut libc::c_void);
    (*pCur).zPrefix = std::ptr::null_mut();
    (*pCur).nPrefix = 0;
    sqlite3_free((*pCur).zLine as *mut libc::c_void);
    (*pCur).zLine = std::ptr::null_mut();
    (*pCur).nLine = 0;
    sqlite3_finalize((*pCur).pStmt);
    (*pCur).pStmt = 0 as *mut sqlite3_stmt;
    (*pCur).j = 0;
}

unsafe extern "C" fn completionClose(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    completionCursorReset(cur as *mut completion_cursor);
    sqlite3_free(cur as *mut libc::c_void);
    return 0;
}
unsafe extern "C" fn completionNext(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut completion_cursor = cur as *mut completion_cursor;
    let mut eNextPhase: i32 = 0;
    let mut iCol: i32 = -1;
    (*pCur).iRowid += 1;
    while (*pCur).ePhase != 11 {
        match (*pCur).ePhase {
            1 => {
                if (*pCur).j >= sqlite3_keyword_count() {
                    (*pCur).zCurrentRow = 0 as *const i8;
                    (*pCur).ePhase = 7 as i32;
                } else {
                    sqlite3_keyword_name((*pCur).j, &mut (*pCur).zCurrentRow, &mut (*pCur).szRow);
                    (*pCur).j += 1;
                }
                iCol = -1;
            }
            7 => {
                if ((*pCur).pStmt).is_null() {
                    sqlite3_prepare_v2(
                        (*pCur).db,
                        b"PRAGMA database_list\0" as *const u8 as *const i8,
                        -1,
                        &mut (*pCur).pStmt,
                        0 as *mut *const i8,
                    );
                }
                iCol = 1;
                eNextPhase = 8;
            }
            8 => {
                if ((*pCur).pStmt).is_null() {
                    let mut pS2: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                    let mut zSql: *mut i8 = std::ptr::null_mut();
                    let mut zSep: *const i8 = b"\0" as *const u8 as *const i8;
                    sqlite3_prepare_v2(
                        (*pCur).db,
                        b"PRAGMA database_list\0" as *const u8 as *const i8,
                        -1,
                        &mut pS2,
                        0 as *mut *const i8,
                    );
                    while sqlite3_step(pS2) == 100 {
                        let mut zDb: *const i8 = sqlite3_column_text(pS2, 1) as *const i8;
                        zSql = sqlite3_mprintf(
                            b"%z%sSELECT name FROM \"%w\".sqlite_schema\0" as *const u8 as *const i8,
                            zSql,
                            zSep,
                            zDb,
                        );
                        if zSql.is_null() {
                            return 7 as i32;
                        }
                        zSep = b" UNION \0" as *const u8 as *const i8;
                    }
                    sqlite3_finalize(pS2);
                    sqlite3_prepare_v2((*pCur).db, zSql, -1, &mut (*pCur).pStmt, 0 as *mut *const i8);
                    sqlite3_free(zSql as *mut libc::c_void);
                }
                iCol = 0;
                eNextPhase = 9;
            }
            9 => {
                if ((*pCur).pStmt).is_null() {
                    let mut pS2_0: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                    let mut zSql_0: *mut i8 = std::ptr::null_mut();
                    let mut zSep_0: *const i8 = b"\0" as *const u8 as *const i8;
                    sqlite3_prepare_v2(
                        (*pCur).db,
                        b"PRAGMA database_list\0" as *const u8 as *const i8,
                        -1,
                        &mut pS2_0,
                        0 as *mut *const i8,
                    );
                    while sqlite3_step(pS2_0) == 100 {
                        let mut zDb_0: *const i8 = sqlite3_column_text(pS2_0, 1) as *const i8;
                        zSql_0 = sqlite3_mprintf(
                            b"%z%sSELECT pti.name FROM \"%w\".sqlite_schema AS sm JOIN pragma_table_info(sm.name,%Q) AS pti WHERE sm.type='table'\0"
                                as *const u8 as *const i8,
                            zSql_0,
                            zSep_0,
                            zDb_0,
                            zDb_0,
                        );
                        if zSql_0.is_null() {
                            return 7 as i32;
                        }
                        zSep_0 = b" UNION \0" as *const u8 as *const i8;
                    }
                    sqlite3_finalize(pS2_0);
                    sqlite3_prepare_v2((*pCur).db, zSql_0, -1, &mut (*pCur).pStmt, 0 as *mut *const i8);
                    sqlite3_free(zSql_0 as *mut libc::c_void);
                }
                iCol = 0;
                eNextPhase = 11;
            }
            _ => {}
        }
        if iCol < 0 {
            if ((*pCur).zCurrentRow).is_null() {
                continue;
            }
        } else if sqlite3_step((*pCur).pStmt) == 100 {
            (*pCur).zCurrentRow = sqlite3_column_text((*pCur).pStmt, iCol) as *const i8;
            (*pCur).szRow = sqlite3_column_bytes((*pCur).pStmt, iCol);
        } else {
            sqlite3_finalize((*pCur).pStmt);
            (*pCur).pStmt = 0 as *mut sqlite3_stmt;
            (*pCur).ePhase = eNextPhase;
            continue;
        }
        if (*pCur).nPrefix == 0 {
            break;
        }
        if (*pCur).nPrefix <= (*pCur).szRow && sqlite3_strnicmp((*pCur).zPrefix, (*pCur).zCurrentRow, (*pCur).nPrefix) == 0 {
            break;
        }
    }
    return 0;
}
unsafe extern "C" fn completionColumn(mut cur: *mut sqlite3_vtab_cursor, mut ctx: *mut sqlite3_context, mut i: i32) -> i32 {
    let mut pCur: *mut completion_cursor = cur as *mut completion_cursor;
    match i {
        0 => {
            sqlite3_result_text(
                ctx,
                (*pCur).zCurrentRow,
                (*pCur).szRow,
                ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
            );
        }
        1 => {
            sqlite3_result_text(
                ctx,
                (*pCur).zPrefix,
                -1,
                ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
            );
        }
        2 => {
            sqlite3_result_text(
                ctx,
                (*pCur).zLine,
                -1,
                ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
            );
        }
        3 => {
            sqlite3_result_int(ctx, (*pCur).ePhase);
        }
        _ => {}
    }
    return 0;
}
unsafe extern "C" fn completionRowid(mut cur: *mut sqlite3_vtab_cursor, mut pRowid: *mut i64) -> i32 {
    let mut pCur: *mut completion_cursor = cur as *mut completion_cursor;
    *pRowid = (*pCur).iRowid;
    return 0;
}
unsafe extern "C" fn completionEof(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut completion_cursor = cur as *mut completion_cursor;
    return ((*pCur).ePhase >= 11) as i32;
}

/*
** This method is called to "rewind" the completion_cursor object back
** to the first row of output.  This method is always called at least
** once prior to any call to completionColumn() or completionRowid() or
** completionEof().
*/
unsafe extern "C" fn completionFilter(
    mut pVtabCursor: *mut sqlite3_vtab_cursor,
    mut idxNum: i32,
    mut _idxStr: *const i8,
    mut _argc: i32,
    mut argv: *mut *mut sqlite3_value,
) -> i32 {
    let mut pCur: *mut completion_cursor = pVtabCursor as *mut completion_cursor;
    let mut iArg: i32 = 0;
    completionCursorReset(pCur);
    if idxNum & 1 != 0 {
        (*pCur).nPrefix = sqlite3_value_bytes(*argv.offset(iArg as isize));
        if (*pCur).nPrefix > 0 {
            (*pCur).zPrefix = sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, sqlite3_value_text(*argv.offset(iArg as isize)));
            if ((*pCur).zPrefix).is_null() {
                return 7 as i32;
            }
        }
        iArg = 1;
    }
    if idxNum & 2 != 0 {
        (*pCur).nLine = sqlite3_value_bytes(*argv.offset(iArg as isize));
        if (*pCur).nLine > 0 {
            (*pCur).zLine = sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, sqlite3_value_text(*argv.offset(iArg as isize)));
            if ((*pCur).zLine).is_null() {
                return 7 as i32;
            }
        }
    }
    if !((*pCur).zLine).is_null() && ((*pCur).zPrefix).is_null() {
        let mut i: i32 = (*pCur).nLine;
        while i > 0
            && ((*((*pCur).zLine).offset((i - 1) as isize) as u8).is_ascii_alphanumeric()
                || *((*pCur).zLine).offset((i - 1) as isize) as i32 == '_' as i32)
        {
            i -= 1;
        }
        (*pCur).nPrefix = (*pCur).nLine - i;
        if (*pCur).nPrefix > 0 {
            (*pCur).zPrefix = sqlite3_mprintf(
                b"%.*s\0" as *const u8 as *const i8,
                (*pCur).nPrefix,
                ((*pCur).zLine).offset(i as isize),
            );
            if ((*pCur).zPrefix).is_null() {
                return 7 as i32;
            }
        }
    }
    (*pCur).iRowid = 0;
    (*pCur).ePhase = 1;
    return completionNext(pVtabCursor);
}
unsafe extern "C" fn completionBestIndex(mut tab: *mut sqlite3_vtab, mut pIdxInfo: *mut sqlite3_index_info) -> i32 {
    // let mut i: i32 = 0;
    let mut idxNum: i32 = 0;
    let mut prefixIdx: i32 = -1;
    let mut wholelineIdx: i32 = -1;
    let mut nArg: i32 = 0;
    let mut pConstraint: *const sqlite3_index_constraint = 0 as *const sqlite3_index_constraint;
    pConstraint = (*pIdxInfo).aConstraint;
    for i in 0..(*pIdxInfo).nConstraint {
        if !((*pConstraint).usable as i32 == 0) {
            if !((*pConstraint).op as i32 != 2) {
                match (*pConstraint).iColumn {
                    1 => {
                        prefixIdx = i;
                        idxNum |= 1;
                    }
                    2 => {
                        wholelineIdx = i;
                        idxNum |= 2;
                    }
                    _ => {}
                }
            }
        }
        pConstraint = pConstraint.offset(1);
    }
    if prefixIdx >= 0 {
        nArg += 1;
        (*((*pIdxInfo).aConstraintUsage).offset(prefixIdx as isize)).argvIndex = nArg;
        (*((*pIdxInfo).aConstraintUsage).offset(prefixIdx as isize)).omit = 1;
    }
    if wholelineIdx >= 0 {
        nArg += 1;
        (*((*pIdxInfo).aConstraintUsage).offset(wholelineIdx as isize)).argvIndex = nArg;
        (*((*pIdxInfo).aConstraintUsage).offset(wholelineIdx as isize)).omit = 1;
    }
    (*pIdxInfo).idxNum = idxNum;
    (*pIdxInfo).estimatedCost = 5000 as f64 - (1000 * nArg) as f64;
    (*pIdxInfo).estimatedRows = (500 - 100 * nArg) as i64;
    return 0;
}
static mut completionModule: sqlite3_module = unsafe {
    {
        let mut init = sqlite3_module {
            iVersion: 0,
            xCreate: None,
            xConnect: Some(completionConnect),
            xBestIndex: Some(completionBestIndex),
            xDisconnect: Some(completionDisconnect),
            xDestroy: None,
            xOpen: Some(completionOpen),
            xClose: Some(completionClose),
            xFilter: Some(completionFilter),
            xNext: Some(completionNext),
            xEof: Some(completionEof),
            xColumn: Some(completionColumn),
            xRowid: Some(completionRowid),
            xUpdate: None,
            xBegin: None,
            xSync: None,
            xCommit: None,
            xRollback: None,
            xFindFunction: None,
            xRename: None,
            xSavepoint: None,
            xRelease: None,
            xRollbackTo: None,
            xShadowName: None,
        };
        init
    }
};
#[no_mangle]
pub unsafe extern "C" fn sqlite3CompletionVtabInit(mut db: *mut sqlite3) -> i32 {
    let mut rc: i32 = sqlite3_create_module(
        db,
        b"completion\0" as *const u8 as *const i8,
        &mut completionModule,
        std::ptr::null_mut(),
    );
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_completion_init(
    mut db: *mut sqlite3,
    mut pzErrMsg: *mut *mut i8,
    mut pApi: *const sqlite3_api_routines,
) -> i32 {
    let rc: i32 = sqlite3CompletionVtabInit(db);
    return rc;
}

static mut apnd_vfs: sqlite3_vfs = sqlite3_vfs {
    iVersion: 3,
    szOsFile: 0,
    mxPathname: 1024,
    pNext: std::ptr::null_mut(),
    zName: b"apndvfs\0" as *const u8 as *const i8,
    pAppData: std::ptr::null_mut(),
    xOpen: Some(apndOpen),
    xDelete: Some(apndDelete),
    xAccess: Some(apndAccess),
    xFullPathname: Some(apndFullPathname),
    xDlOpen: Some(apndDlOpen),
    xDlError: Some(apndDlError),
    xDlSym: Some(apndDlSym),
    xDlClose: Some(apndDlClose),
    xRandomness: Some(apndRandomness),
    xSleep: Some(apndSleep),
    xCurrentTime: Some(apndCurrentTime),
    xGetLastError: Some(apndGetLastError),
    xCurrentTimeInt64: Some(apndCurrentTimeInt64),
    xSetSystemCall: Some(apndSetSystemCall),
    xGetSystemCall: Some(apndGetSystemCall),
    xNextSystemCall: Some(apndNextSystemCall),
};

static apnd_io_methods: sqlite3_io_methods = sqlite3_io_methods {
    iVersion: 3,
    xClose: Some(apndClose),
    xRead: Some(apndRead),
    xWrite: Some(apndWrite),
    xTruncate: Some(apndTruncate),
    xSync: Some(apndSync),
    xFileSize: Some(apndFileSize),
    xLock: Some(apndLock),
    xUnlock: Some(apndUnlock),
    xCheckReservedLock: Some(apndCheckReservedLock),
    xFileControl: Some(apndFileControl),
    xSectorSize: Some(apndSectorSize),
    xDeviceCharacteristics: Some(apndDeviceCharacteristics),
    xShmMap: Some(apndShmMap),
    xShmLock: Some(apndShmLock),
    xShmBarrier: Some(apndShmBarrier),
    xShmUnmap: Some(apndShmUnmap),
    xFetch: Some(apndFetch),
    xUnfetch: Some(apndUnfetch),
};

unsafe extern "C" fn apndClose(mut pFile: *mut sqlite3_file) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xClose).expect("non-null function pointer")(pFile);
}
unsafe extern "C" fn apndRead(mut pFile: *mut sqlite3_file, mut zBuf: *mut libc::c_void, mut iAmt: i32, mut iOfst: i64) -> i32 {
    let mut paf: *mut ApndFile = pFile as *mut ApndFile;
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xRead).expect("non-null function pointer")(pFile, zBuf, iAmt, (*paf).iPgOne + iOfst);
}
unsafe extern "C" fn apndWriteMark(mut paf: *mut ApndFile, mut pFile: *mut sqlite3_file, mut iWriteEnd: i64) -> i32 {
    let mut iPgOne: i64 = (*paf).iPgOne;
    let mut a: [u8; 25] = [0; 25];
    let mut i: i32 = 8;
    let mut rc: i32 = 0;
    assert!(pFile == paf.offset(1) as *mut sqlite3_file);
    memcpy(
        a.as_mut_ptr() as *mut libc::c_void,
        b"Start-Of-SQLite3-\0" as *const u8 as *const i8 as *const libc::c_void,
        17 as u64,
    );
    loop {
        i -= 1;
        if !(i >= 0) {
            break;
        }
        a[(17 + i) as usize] = (iPgOne & 0xff as i32 as i64) as u8;
        iPgOne >>= 8;
    }
    iWriteEnd += (*paf).iPgOne;
    rc = ((*(*pFile).pMethods).xWrite).expect("non-null function pointer")(pFile, a.as_mut_ptr() as *const libc::c_void, 17 + 8, iWriteEnd);
    if 0 == rc {
        (*paf).iMark = iWriteEnd;
    }
    return rc;
}
unsafe extern "C" fn apndWrite(mut pFile: *mut sqlite3_file, mut zBuf: *const libc::c_void, mut iAmt: i32, mut iOfst: i64) -> i32 {
    let mut paf: *mut ApndFile = pFile as *mut ApndFile;
    let mut iWriteEnd: i64 = iOfst + iAmt as i64;
    if iWriteEnd >= 0x40000000 as i64 {
        return 13;
    }
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    if (*paf).iMark < 0 || (*paf).iPgOne + iWriteEnd > (*paf).iMark {
        let mut rc: i32 = apndWriteMark(paf, pFile, iWriteEnd);
        if 0 != rc {
            return rc;
        }
    }
    return ((*(*pFile).pMethods).xWrite).expect("non-null function pointer")(pFile, zBuf, iAmt, (*paf).iPgOne + iOfst);
}
unsafe extern "C" fn apndTruncate(mut pFile: *mut sqlite3_file, mut size: i64) -> i32 {
    let mut paf: *mut ApndFile = pFile as *mut ApndFile;
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    if 0 != apndWriteMark(paf, pFile, size) {
        return 10;
    }
    return ((*(*pFile).pMethods).xTruncate).expect("non-null function pointer")(pFile, (*paf).iMark + (17 + 8) as i64);
}
unsafe extern "C" fn apndSync(mut pFile: *mut sqlite3_file, mut flags: i32) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xSync).expect("non-null function pointer")(pFile, flags);
}
unsafe extern "C" fn apndFileSize(mut pFile: *mut sqlite3_file, mut pSize: *mut i64) -> i32 {
    let mut paf: *mut ApndFile = pFile as *mut ApndFile;
    *pSize = if (*paf).iMark >= 0 { (*paf).iMark - (*paf).iPgOne } else { 0 };
    return 0;
}
unsafe extern "C" fn apndLock(mut pFile: *mut sqlite3_file, mut eLock: i32) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xLock).expect("non-null function pointer")(pFile, eLock);
}
unsafe extern "C" fn apndUnlock(mut pFile: *mut sqlite3_file, mut eLock: i32) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xUnlock).expect("non-null function pointer")(pFile, eLock);
}
unsafe extern "C" fn apndCheckReservedLock(mut pFile: *mut sqlite3_file, mut pResOut: *mut i32) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xCheckReservedLock).expect("non-null function pointer")(pFile, pResOut);
}
unsafe extern "C" fn apndFileControl(mut pFile: *mut sqlite3_file, mut op: i32, mut pArg: *mut libc::c_void) -> i32 {
    let mut paf: *mut ApndFile = pFile as *mut ApndFile;
    let mut rc: i32 = 0;
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    if op == 5 {
        *(pArg as *mut i64) += (*paf).iPgOne;
    }
    rc = ((*(*pFile).pMethods).xFileControl).expect("non-null function pointer")(pFile, op, pArg);
    if rc == 0 && op == 12 {
        *(pArg as *mut *mut i8) = sqlite3_mprintf(b"apnd(%lld)/%z\0" as *const u8 as *const i8, (*paf).iPgOne, *(pArg as *mut *mut i8));
    }
    return rc;
}
unsafe extern "C" fn apndSectorSize(mut pFile: *mut sqlite3_file) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xSectorSize).expect("non-null function pointer")(pFile);
}
unsafe extern "C" fn apndDeviceCharacteristics(mut pFile: *mut sqlite3_file) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xDeviceCharacteristics).expect("non-null function pointer")(pFile);
}
unsafe extern "C" fn apndShmMap(
    mut pFile: *mut sqlite3_file,
    mut iPg: i32,
    mut pgsz: i32,
    mut bExtend: i32,
    mut pp: *mut *mut libc::c_void,
) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xShmMap).expect("non-null function pointer")(pFile, iPg, pgsz, bExtend, pp);
}
unsafe extern "C" fn apndShmLock(mut pFile: *mut sqlite3_file, mut offset: i32, mut n: i32, mut flags: i32) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xShmLock).expect("non-null function pointer")(pFile, offset, n, flags);
}
unsafe extern "C" fn apndShmBarrier(mut pFile: *mut sqlite3_file) {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    ((*(*pFile).pMethods).xShmBarrier).expect("non-null function pointer")(pFile);
}
unsafe extern "C" fn apndShmUnmap(mut pFile: *mut sqlite3_file, mut deleteFlag: i32) -> i32 {
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xShmUnmap).expect("non-null function pointer")(pFile, deleteFlag);
}
unsafe extern "C" fn apndFetch(mut pFile: *mut sqlite3_file, mut iOfst: i64, mut iAmt: i32, mut pp: *mut *mut libc::c_void) -> i32 {
    let mut p: *mut ApndFile = pFile as *mut ApndFile;
    if (*p).iMark < 0 || iOfst + iAmt as i64 > (*p).iMark {
        return 10;
    }
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xFetch).expect("non-null function pointer")(pFile, iOfst + (*p).iPgOne, iAmt, pp);
}
unsafe extern "C" fn apndUnfetch(mut pFile: *mut sqlite3_file, mut iOfst: i64, mut pPage: *mut libc::c_void) -> i32 {
    let mut p: *mut ApndFile = pFile as *mut ApndFile;
    pFile = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    return ((*(*pFile).pMethods).xUnfetch).expect("non-null function pointer")(pFile, iOfst + (*p).iPgOne, pPage);
}
unsafe extern "C" fn apndReadMark(mut sz: i64, mut pFile: *mut sqlite3_file) -> i64 {
    let mut rc: i32 = 0;
    let mut iMark: i64 = 0;
    let mut msbs: i32 = 8 * (8 - 1);
    let mut a: [u8; 25] = [0; 25];
    if (17 + 8) as i64 != sz & 0x1ff as i32 as i64 {
        return -1;
    }
    rc = ((*(*pFile).pMethods).xRead).expect("non-null function pointer")(
        pFile,
        a.as_mut_ptr() as *mut libc::c_void,
        17 + 8,
        sz - (17 + 8) as i64,
    );
    if rc != 0 {
        return -1;
    }
    if memcmp(
        a.as_mut_ptr() as *const libc::c_void,
        b"Start-Of-SQLite3-\0" as *const u8 as *const i8 as *const libc::c_void,
        17 as u64,
    ) != 0
    {
        return -1;
    }
    iMark = ((a[17] as i32 & 0x7f as i32) as i64) << msbs;
    for i in 1..8 {
        msbs -= 8;
        iMark |= (a[(17 + i) as usize] as i64) << msbs;
    }
    if iMark > sz - (17 + 8) as i64 - 512 as i64 {
        return -1;
    }
    if iMark & 0x1ff as i32 as i64 != 0 {
        return -1;
    }
    return iMark;
}
static mut apvfsSqliteHdr: [i8; 16] = unsafe { *::std::mem::transmute::<&[u8; 16], &[i8; 16]>(b"SQLite format 3\0") };
unsafe fn apndIsAppendvfsDatabase(mut sz: i64, mut pFile: *mut sqlite3_file) -> i32 {
    let mut rc: i32 = 0;
    let mut zHdr: [i8; 16] = [0; 16];
    let mut iMark: i64 = apndReadMark(sz, pFile);
    if iMark >= 0 {
        rc = ((*(*pFile).pMethods).xRead).expect("non-null function pointer")(
            pFile,
            zHdr.as_mut_ptr() as *mut libc::c_void,
            ::std::mem::size_of::<[i8; 16]>() as u64 as i32,
            iMark,
        );
        if 0 == rc
            && memcmp(
                zHdr.as_mut_ptr() as *const libc::c_void,
                apvfsSqliteHdr.as_ptr() as *const libc::c_void,
                ::std::mem::size_of::<[i8; 16]>() as u64,
            ) == 0
            && sz & 0x1ff as i32 as i64 == (17 + 8) as i64
            && sz >= (512 + (17 + 8)) as i64
        {
            return 1;
        }
    }
    return 0;
}
unsafe extern "C" fn apndIsOrdinaryDatabaseFile(mut sz: i64, mut pFile: *mut sqlite3_file) -> i32 {
    let mut zHdr: [i8; 16] = [0; 16];
    if apndIsAppendvfsDatabase(sz, pFile) != 0
        || sz & 0x1ff as i32 as i64 != 0
        || 0 != ((*(*pFile).pMethods).xRead).expect("non-null function pointer")(
            pFile,
            zHdr.as_mut_ptr() as *mut libc::c_void,
            ::std::mem::size_of::<[i8; 16]>() as u64 as i32,
            0,
        )
        || memcmp(
            zHdr.as_mut_ptr() as *const libc::c_void,
            apvfsSqliteHdr.as_ptr() as *const libc::c_void,
            ::std::mem::size_of::<[i8; 16]>() as u64,
        ) != 0
    {
        return 0;
    } else {
        return 1;
    };
}
unsafe extern "C" fn apndOpen(
    mut pApndVfs: *mut sqlite3_vfs,
    mut zName: *const i8,
    mut pFile: *mut sqlite3_file,
    mut flags: i32,
    mut pOutFlags: *mut i32,
) -> i32 {
    let mut pApndFile: *mut ApndFile = pFile as *mut ApndFile;
    let mut pBaseFile: *mut sqlite3_file = (pFile as *mut ApndFile).offset(1) as *mut sqlite3_file;
    let mut pBaseVfs: *mut sqlite3_vfs = (*pApndVfs).pAppData as *mut sqlite3_vfs;
    let mut rc: i32 = 0;
    let mut sz: i64 = 0;
    if flags & 0x100 == 0 {
        return ((*pBaseVfs).xOpen).expect("non-null function pointer")(pBaseVfs, zName, pFile, flags, pOutFlags);
    }
    memset(pApndFile as *mut libc::c_void, 0, ::std::mem::size_of::<ApndFile>() as u64);
    (*pFile).pMethods = &apnd_io_methods;
    (*pApndFile).iMark = -1;
    rc = ((*pBaseVfs).xOpen).expect("non-null function pointer")(pBaseVfs, zName, pBaseFile, flags, pOutFlags);
    if rc == 0 {
        rc = ((*(*pBaseFile).pMethods).xFileSize).expect("non-null function pointer")(pBaseFile, &mut sz);
        if rc != 0 {
            ((*(*pBaseFile).pMethods).xClose).expect("non-null function pointer")(pBaseFile);
        }
    }
    if rc != 0 {
        (*pFile).pMethods = std::ptr::null();
        return rc;
    }
    if apndIsOrdinaryDatabaseFile(sz, pBaseFile) != 0 {
        memmove(
            pApndFile as *mut libc::c_void,
            pBaseFile as *const libc::c_void,
            (*pBaseVfs).szOsFile as u64,
        );
        return 0;
    }
    (*pApndFile).iPgOne = apndReadMark(sz, pFile);
    if (*pApndFile).iPgOne >= 0 {
        (*pApndFile).iMark = sz - (17 + 8) as i64;
        return 0;
    }
    if flags & 0x4 as i32 == 0 {
        ((*(*pBaseFile).pMethods).xClose).expect("non-null function pointer")(pBaseFile);
        rc = 14 as i32;
        (*pFile).pMethods = std::ptr::null();
    } else {
        (*pApndFile).iPgOne = sz + (4096 - 1) as i64 & !((4096 - 1) as i64);
    }
    return rc;
}
unsafe extern "C" fn apndDelete(mut pVfs: *mut sqlite3_vfs, mut zPath: *const i8, mut dirSync: i32) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xDelete).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        zPath,
        dirSync,
    );
}
unsafe extern "C" fn apndAccess(mut pVfs: *mut sqlite3_vfs, mut zPath: *const i8, mut flags: i32, mut pResOut: *mut i32) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xAccess).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        zPath,
        flags,
        pResOut,
    );
}
unsafe extern "C" fn apndFullPathname(mut pVfs: *mut sqlite3_vfs, mut zPath: *const i8, mut nOut: i32, mut zOut: *mut i8) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xFullPathname).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        zPath,
        nOut,
        zOut,
    );
}
unsafe extern "C" fn apndDlOpen(mut pVfs: *mut sqlite3_vfs, mut zPath: *const i8) -> *mut libc::c_void {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xDlOpen).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        zPath,
    );
}
unsafe extern "C" fn apndDlError(mut pVfs: *mut sqlite3_vfs, mut nByte: i32, mut zErrMsg: *mut i8) {
    ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xDlError).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        nByte,
        zErrMsg,
    );
}
unsafe extern "C" fn apndDlSym(
    mut pVfs: *mut sqlite3_vfs,
    mut p: *mut libc::c_void,
    mut zSym: *const i8,
) -> Option<unsafe extern "C" fn() -> ()> {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xDlSym).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        p,
        zSym,
    );
}
unsafe extern "C" fn apndDlClose(mut pVfs: *mut sqlite3_vfs, mut pHandle: *mut libc::c_void) {
    ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xDlClose).expect("non-null function pointer")((*pVfs).pAppData as *mut sqlite3_vfs, pHandle);
}
unsafe extern "C" fn apndRandomness(mut pVfs: *mut sqlite3_vfs, mut nByte: i32, mut zBufOut: *mut i8) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xRandomness).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        nByte,
        zBufOut,
    );
}
unsafe extern "C" fn apndSleep(mut pVfs: *mut sqlite3_vfs, mut nMicro: i32) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xSleep).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        nMicro,
    );
}
unsafe extern "C" fn apndCurrentTime(mut pVfs: *mut sqlite3_vfs, mut pTimeOut: *mut f64) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xCurrentTime).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        pTimeOut,
    );
}
unsafe extern "C" fn apndGetLastError(mut pVfs: *mut sqlite3_vfs, mut a: i32, mut b: *mut i8) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xGetLastError).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        a,
        b,
    );
}
unsafe extern "C" fn apndCurrentTimeInt64(mut pVfs: *mut sqlite3_vfs, mut p: *mut i64) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xCurrentTimeInt64).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        p,
    );
}
unsafe extern "C" fn apndSetSystemCall(mut pVfs: *mut sqlite3_vfs, mut zName: *const i8, mut pCall: sqlite3_syscall_ptr) -> i32 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xSetSystemCall).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        zName,
        pCall,
    );
}
unsafe extern "C" fn apndGetSystemCall(mut pVfs: *mut sqlite3_vfs, mut zName: *const i8) -> sqlite3_syscall_ptr {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xGetSystemCall).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        zName,
    );
}
unsafe extern "C" fn apndNextSystemCall(mut pVfs: *mut sqlite3_vfs, mut zName: *const i8) -> *const i8 {
    return ((*((*pVfs).pAppData as *mut sqlite3_vfs)).xNextSystemCall).expect("non-null function pointer")(
        (*pVfs).pAppData as *mut sqlite3_vfs,
        zName,
    );
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_appendvfs_init(
    mut db: *mut sqlite3,
    mut pzErrMsg: *mut *mut i8,
    mut pApi: *const sqlite3_api_routines,
) -> i32 {
    let mut rc: i32 = 0;
    let mut pOrig: *mut sqlite3_vfs = std::ptr::null_mut();
    pOrig = sqlite3_vfs_find(0 as *const i8);
    if pOrig.is_null() {
        return 1;
    }
    apnd_vfs.iVersion = (*pOrig).iVersion;
    apnd_vfs.pAppData = pOrig as *mut libc::c_void;
    apnd_vfs.szOsFile = ((*pOrig).szOsFile as u64).wrapping_add(::std::mem::size_of::<ApndFile>() as u64) as i32;
    rc = sqlite3_vfs_register(&mut apnd_vfs, 0);
    if rc == 0 {
        rc = 0 | (1) << 8;
    }
    return rc;
}
static mut memtraceBase: sqlite3_mem_methods = sqlite3_mem_methods {
    xMalloc: None,
    xFree: None,
    xRealloc: None,
    xSize: None,
    xRoundup: None,
    xInit: None,
    xShutdown: None,
    pAppData: std::ptr::null_mut(),
};
static mut memtraceOut: *mut FILE = 0 as *const FILE as *mut FILE;
unsafe extern "C" fn memtraceMalloc(mut n: i32) -> *mut libc::c_void {
    if !memtraceOut.is_null() {
        fprintf(
            memtraceOut,
            b"MEMTRACE: allocate %d bytes\n\0" as *const u8 as *const i8,
            (memtraceBase.xRoundup).expect("non-null function pointer")(n),
        );
    }
    return (memtraceBase.xMalloc).expect("non-null function pointer")(n);
}
unsafe extern "C" fn memtraceFree(mut p: *mut libc::c_void) {
    if p.is_null() {
        return;
    }
    if !memtraceOut.is_null() {
        fprintf(
            memtraceOut,
            b"MEMTRACE: free %d bytes\n\0" as *const u8 as *const i8,
            (memtraceBase.xSize).expect("non-null function pointer")(p),
        );
    }
    (memtraceBase.xFree).expect("non-null function pointer")(p);
}
unsafe extern "C" fn memtraceRealloc(mut p: *mut libc::c_void, mut n: i32) -> *mut libc::c_void {
    if p.is_null() {
        return memtraceMalloc(n);
    }
    if n == 0 {
        memtraceFree(p);
        return std::ptr::null_mut();
    }
    if !memtraceOut.is_null() {
        fprintf(
            memtraceOut,
            b"MEMTRACE: resize %d -> %d bytes\n\0" as *const u8 as *const i8,
            (memtraceBase.xSize).expect("non-null function pointer")(p),
            (memtraceBase.xRoundup).expect("non-null function pointer")(n),
        );
    }
    return (memtraceBase.xRealloc).expect("non-null function pointer")(p, n);
}
unsafe extern "C" fn memtraceSize(mut p: *mut libc::c_void) -> i32 {
    return (memtraceBase.xSize).expect("non-null function pointer")(p);
}
unsafe extern "C" fn memtraceRoundup(mut n: i32) -> i32 {
    return (memtraceBase.xRoundup).expect("non-null function pointer")(n);
}
unsafe extern "C" fn memtraceInit(mut p: *mut libc::c_void) -> i32 {
    return (memtraceBase.xInit).expect("non-null function pointer")(p);
}
unsafe extern "C" fn memtraceShutdown(mut p: *mut libc::c_void) {
    (memtraceBase.xShutdown).expect("non-null function pointer")(p);
}
static mut ersaztMethods: sqlite3_mem_methods = unsafe {
    sqlite3_mem_methods {
        xMalloc: Some(memtraceMalloc),
        xFree: Some(memtraceFree),
        xRealloc: Some(memtraceRealloc),
        xSize: Some(memtraceSize),
        xRoundup: Some(memtraceRoundup),
        xInit: Some(memtraceInit),
        xShutdown: Some(memtraceShutdown),
        pAppData: std::ptr::null_mut(),
    }
};
#[no_mangle]
pub unsafe extern "C" fn sqlite3MemTraceActivate(mut out: *mut FILE) -> i32 {
    let mut rc: i32 = 0;
    if (memtraceBase.xMalloc).is_none() {
        rc = sqlite3_config(5, &mut memtraceBase as *mut sqlite3_mem_methods);
        if rc == 0 {
            rc = sqlite3_config(4 as i32, &mut ersaztMethods as *mut sqlite3_mem_methods);
        }
    }
    memtraceOut = out;
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3MemTraceDeactivate() -> i32 {
    let mut rc: i32 = 0;
    if (memtraceBase.xMalloc).is_some() {
        rc = sqlite3_config(4 as i32, &mut memtraceBase as *mut sqlite3_mem_methods);
        if rc == 0 {
            memset(
                &mut memtraceBase as *mut sqlite3_mem_methods as *mut libc::c_void,
                0,
                ::std::mem::size_of::<sqlite3_mem_methods>() as u64,
            );
        }
    }
    memtraceOut = 0 as *mut FILE;
    return rc;
}
unsafe extern "C" fn uintCollFunc(
    mut notUsed: *mut libc::c_void,
    mut nKey1: i32,
    mut pKey1: *const libc::c_void,
    mut nKey2: i32,
    mut pKey2: *const libc::c_void,
) -> i32 {
    let mut zA: *const u8 = pKey1 as *const u8;
    let mut zB: *const u8 = pKey2 as *const u8;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut x: i32 = 0;
    while i < nKey1 && j < nKey2 {
        x = *zA.offset(i as isize) as i32 - *zB.offset(j as isize) as i32;
        if (*zA.offset(i as isize)).is_ascii_digit() {
            let mut k: i32 = 0;
            if (*zB.offset(j as isize)).is_ascii_digit() == false {
                return x;
            }
            while i < nKey1 && *zA.offset(i as isize) as i32 == '0' as i32 {
                i += 1;
            }
            while j < nKey2 && *zB.offset(j as isize) as i32 == '0' as i32 {
                j += 1;
            }
            k = 0;
            while i + k < nKey1
                && (*zA.offset((i + k) as isize)).is_ascii_digit()
                && j + k < nKey2
                && (*zB.offset((j + k) as isize)).is_ascii_digit()
            {
                k += 1;
            }
            if i + k < nKey1 && (*zA.offset((i + k) as isize)).is_ascii_digit() {
                return 1;
            } else {
                if j + k < nKey2 && (*zB.offset((j + k) as isize)).is_ascii_digit() {
                    return -1;
                } else {
                    x = memcmp(
                        zA.offset(i as isize) as *const libc::c_void,
                        zB.offset(j as isize) as *const libc::c_void,
                        k as u64,
                    );
                    if x != 0 {
                        return x;
                    }
                    i += k;
                    j += k;
                }
            }
        } else if x != 0 {
            return x;
        } else {
            i += 1;
            j += 1;
        }
    }
    return nKey1 - i - (nKey2 - j);
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_uint_init(mut db: *mut sqlite3, mut pzErrMsg: *mut *mut i8, mut pApi: *const sqlite3_api_routines) -> i32 {
    return sqlite3_create_collation(db, b"uint\0" as *const u8 as *const i8, 1, std::ptr::null_mut(), Some(uintCollFunc));
}
unsafe extern "C" fn decimal_clear(mut p: *mut Decimal) {
    sqlite3_free((*p).a as *mut libc::c_void);
}
unsafe extern "C" fn decimal_free(mut p: *mut Decimal) {
    if !p.is_null() {
        decimal_clear(p);
        sqlite3_free(p as *mut libc::c_void);
    }
}
unsafe extern "C" fn decimal_new(
    mut pCtx: *mut sqlite3_context,
    mut pIn: *mut sqlite3_value,
    mut nAlt: i32,
    mut zAlt: *const u8,
) -> *mut Decimal {
    let mut current_block: u64;
    let mut p: *mut Decimal = 0 as *mut Decimal;
    let mut n: i32 = 0;
    let mut i: i32 = 0;
    let mut zIn: *const u8 = std::ptr::null();
    let mut iExp: i32 = 0;
    p = sqlite3_malloc(::std::mem::size_of::<Decimal>() as u64 as i32) as *mut Decimal;
    if !p.is_null() {
        (*p).sign = 0;
        (*p).oom = 0;
        (*p).isInit = 1 as i8;
        (*p).isNull = 0;
        (*p).nDigit = 0;
        (*p).nFrac = 0;
        if !zAlt.is_null() {
            n = nAlt;
            zIn = zAlt;
        } else {
            if sqlite3_value_type(pIn) == 5 {
                (*p).a = 0 as *mut libc::c_schar;
                (*p).isNull = 1 as i8;
                return p;
            }
            n = sqlite3_value_bytes(pIn);
            zIn = sqlite3_value_text(pIn);
        }
        (*p).a = sqlite3_malloc64((n + 1) as u64) as *mut libc::c_schar;
        if !((*p).a).is_null() {
            i = 0;
            while (*zIn.offset(i as isize)).is_ascii_whitespace() {
                i += 1;
            }
            if *zIn.offset(i as isize) as i32 == '-' as i32 {
                (*p).sign = 1 as i8;
                i += 1;
            } else if *zIn.offset(i as isize) as i32 == '+' as i32 {
                i += 1;
            }
            while i < n && *zIn.offset(i as isize) as i32 == '0' as i32 {
                i += 1;
            }
            while i < n {
                let mut c: i8 = *zIn.offset(i as isize) as i8;
                if c as i32 >= '0' as i32 && c as i32 <= '9' as i32 {
                    *((*p).a).offset((*p).nDigit as isize) = (c as i32 - '0' as i32) as libc::c_schar;
                    (*p).nDigit += 1;
                } else if c as i32 == '.' as i32 {
                    (*p).nFrac = (*p).nDigit + 1;
                } else if c as i32 == 'e' as i32 || c as i32 == 'E' as i32 {
                    let mut j: i32 = i + 1;
                    let mut neg: i32 = 0;
                    if j >= n {
                        break;
                    }
                    if *zIn.offset(j as isize) as i32 == '-' as i32 {
                        neg = 1;
                        j += 1;
                    } else if *zIn.offset(j as isize) as i32 == '+' as i32 {
                        j += 1;
                    }
                    while j < n && iExp < 1000000 {
                        if *zIn.offset(j as isize) as i32 >= '0' as i32 && *zIn.offset(j as isize) as i32 <= '9' as i32 {
                            iExp = iExp * 10 + *zIn.offset(j as isize) as i32 - '0' as i32;
                        }
                        j += 1;
                    }
                    if neg != 0 {
                        iExp = -iExp;
                    }
                    break;
                }
                i += 1;
            }
            if (*p).nFrac != 0 {
                (*p).nFrac = (*p).nDigit - ((*p).nFrac - 1);
            }
            if iExp > 0 {
                if (*p).nFrac > 0 {
                    if iExp <= (*p).nFrac {
                        (*p).nFrac -= iExp;
                        iExp = 0;
                    } else {
                        iExp -= (*p).nFrac;
                        (*p).nFrac = 0;
                    }
                }
                if iExp > 0 {
                    (*p).a = sqlite3_realloc64((*p).a as *mut libc::c_void, ((*p).nDigit + iExp + 1) as u64) as *mut libc::c_schar;
                    if ((*p).a).is_null() {
                        current_block = 10261281462084195842;
                    } else {
                        memset(((*p).a).offset((*p).nDigit as isize) as *mut libc::c_void, 0, iExp as u64);
                        (*p).nDigit += iExp;
                        current_block = 17995254032144898061;
                    }
                } else {
                    current_block = 17995254032144898061;
                }
            } else if iExp < 0 {
                let mut nExtra: i32 = 0;
                iExp = -iExp;
                nExtra = (*p).nDigit - (*p).nFrac - 1;
                if nExtra != 0 {
                    if nExtra >= iExp {
                        (*p).nFrac += iExp;
                        iExp = 0;
                    } else {
                        iExp -= nExtra;
                        (*p).nFrac = (*p).nDigit - 1;
                    }
                }
                if iExp > 0 {
                    (*p).a = sqlite3_realloc64((*p).a as *mut libc::c_void, ((*p).nDigit + iExp + 1) as u64) as *mut libc::c_schar;
                    if ((*p).a).is_null() {
                        current_block = 10261281462084195842;
                    } else {
                        memmove(
                            ((*p).a).offset(iExp as isize) as *mut libc::c_void,
                            (*p).a as *const libc::c_void,
                            (*p).nDigit as u64,
                        );
                        memset((*p).a as *mut libc::c_void, 0, iExp as u64);
                        (*p).nDigit += iExp;
                        (*p).nFrac += iExp;
                        current_block = 17995254032144898061;
                    }
                } else {
                    current_block = 17995254032144898061;
                }
            } else {
                current_block = 17995254032144898061;
            }
            match current_block {
                10261281462084195842 => {}
                _ => return p,
            }
        }
    }
    if !pCtx.is_null() {
        sqlite3_result_error_nomem(pCtx);
    }
    sqlite3_free(p as *mut libc::c_void);
    return 0 as *mut Decimal;
}
unsafe extern "C" fn decimal_result(mut pCtx: *mut sqlite3_context, mut p: *mut Decimal) {
    let mut z: *mut i8 = std::ptr::null_mut();
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut n: i32 = 0;
    if p.is_null() || (*p).oom as i32 != 0 {
        sqlite3_result_error_nomem(pCtx);
        return;
    }
    if (*p).isNull != 0 {
        sqlite3_result_null(pCtx);
        return;
    }
    z = sqlite3_malloc((*p).nDigit + 4) as *mut i8;
    if z.is_null() {
        sqlite3_result_error_nomem(pCtx);
        return;
    }
    i = 0;
    if (*p).nDigit == 0 || (*p).nDigit == 1 && *((*p).a).offset(0) as i32 == 0 {
        (*p).sign = 0;
    }
    if (*p).sign != 0 {
        *z.offset(0) = '-' as i32 as i8;
        i = 1;
    }
    n = (*p).nDigit - (*p).nFrac;
    if n <= 0 {
        *z.offset(i as isize) = '0' as i32 as i8;
        i += 1;
    }
    j = 0;
    while n > 1 && *((*p).a).offset(j as isize) as i32 == 0 {
        j += 1;
        n -= 1;
    }
    while n > 0 {
        *z.offset(i as isize) = (*((*p).a).offset(j as isize) as i32 + '0' as i32) as i8;
        i += 1;
        j += 1;
        n -= 1;
    }
    if (*p).nFrac != 0 {
        *z.offset(i as isize) = '.' as i32 as i8;
        i += 1;
        loop {
            *z.offset(i as isize) = (*((*p).a).offset(j as isize) as i32 + '0' as i32) as i8;
            i += 1;
            j += 1;
            if !(j < (*p).nDigit) {
                break;
            }
        }
    }
    *z.offset(i as isize) = 0;
    sqlite3_result_text(pCtx, z, i, Some(sqlite3_free));
}

unsafe extern "C" fn decimalFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut p: *mut Decimal = decimal_new(context, *argv.offset(0), 0, std::ptr::null());
    decimal_result(context, p);
    decimal_free(p);
}
unsafe extern "C" fn decimal_cmp(mut pA: *const Decimal, mut pB: *const Decimal) -> i32 {
    let mut nASig: i32 = 0;
    let mut nBSig: i32 = 0;
    let mut rc: i32 = 0;
    let mut n: i32 = 0;
    if (*pA).sign as i32 != (*pB).sign as i32 {
        return if (*pA).sign as i32 != 0 { -1 } else { 1 };
    }
    if (*pA).sign != 0 {
        let mut pTemp: *const Decimal = pA;
        pA = pB;
        pB = pTemp;
    }
    nASig = (*pA).nDigit - (*pA).nFrac;
    nBSig = (*pB).nDigit - (*pB).nFrac;
    if nASig != nBSig {
        return nASig - nBSig;
    }
    n = (*pA).nDigit;
    if n > (*pB).nDigit {
        n = (*pB).nDigit;
    }
    rc = memcmp((*pA).a as *const libc::c_void, (*pB).a as *const libc::c_void, n as u64);
    if rc == 0 {
        rc = (*pA).nDigit - (*pB).nDigit;
    }
    return rc;
}
unsafe extern "C" fn decimalCmpFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut pA: *mut Decimal = 0 as *mut Decimal;
    let mut pB: *mut Decimal = 0 as *mut Decimal;
    let mut rc: i32 = 0;
    pA = decimal_new(context, *argv.offset(0), 0, std::ptr::null());
    if !(pA.is_null() || (*pA).isNull as i32 != 0) {
        pB = decimal_new(context, *argv.offset(1), 0, std::ptr::null());
        if !(pB.is_null() || (*pB).isNull as i32 != 0) {
            rc = decimal_cmp(pA, pB);
            if rc < 0 {
                rc = -1;
            } else if rc > 0 {
                rc = 1;
            }
            sqlite3_result_int(context, rc);
        }
    }
    decimal_free(pA);
    decimal_free(pB);
}
unsafe extern "C" fn decimal_expand(p: *mut Decimal, nDigit: i32, nFrac: i32) {
    if p.is_null() {
        return;
    }
    let nAddFrac = nFrac - (*p).nFrac;
    let nAddSig = nDigit - (*p).nDigit - nAddFrac;
    if nAddFrac == 0 && nAddSig == 0 {
        return;
    }
    (*p).a = sqlite3_realloc64((*p).a as *mut libc::c_void, (nDigit + 1) as u64) as *mut libc::c_schar;
    if ((*p).a).is_null() {
        (*p).oom = 1 as i8;
        return;
    }
    if nAddSig != 0 {
        memmove(
            ((*p).a).offset(nAddSig as isize) as *mut libc::c_void,
            (*p).a as *const libc::c_void,
            (*p).nDigit as u64,
        );
        memset((*p).a as *mut libc::c_void, 0, nAddSig as u64);
        (*p).nDigit += nAddSig;
    }
    if nAddFrac != 0 {
        memset(((*p).a).offset((*p).nDigit as isize) as *mut libc::c_void, 0, nAddFrac as u64);
        (*p).nDigit += nAddFrac;
        (*p).nFrac += nAddFrac;
    }
}
unsafe extern "C" fn decimal_add(mut pA: *mut Decimal, mut pB: *mut Decimal) {
    let mut nSig: i32 = 0;
    let mut nFrac: i32 = 0;
    let mut nDigit: i32 = 0;
    let mut i: i32 = 0;
    let mut rc: i32 = 0;
    if pA.is_null() {
        return;
    }
    if (*pA).oom as i32 != 0 || pB.is_null() || (*pB).oom as i32 != 0 {
        (*pA).oom = 1 as i8;
        return;
    }
    if (*pA).isNull as i32 != 0 || (*pB).isNull as i32 != 0 {
        (*pA).isNull = 1 as i8;
        return;
    }
    nSig = (*pA).nDigit - (*pA).nFrac;
    if nSig != 0 && *((*pA).a).offset(0) as i32 == 0 {
        nSig -= 1;
    }
    if nSig < (*pB).nDigit - (*pB).nFrac {
        nSig = (*pB).nDigit - (*pB).nFrac;
    }
    nFrac = (*pA).nFrac;
    if nFrac < (*pB).nFrac {
        nFrac = (*pB).nFrac;
    }
    nDigit = nSig + nFrac + 1;
    decimal_expand(pA, nDigit, nFrac);
    decimal_expand(pB, nDigit, nFrac);
    if (*pA).oom as i32 != 0 || (*pB).oom as i32 != 0 {
        (*pA).oom = 1 as i8;
    } else if (*pA).sign as i32 == (*pB).sign as i32 {
        let mut carry: i32 = 0;
        i = nDigit - 1;
        while i >= 0 {
            let mut x: i32 = *((*pA).a).offset(i as isize) as i32 + *((*pB).a).offset(i as isize) as i32 + carry;
            if x >= 10 {
                carry = 1;
                *((*pA).a).offset(i as isize) = (x - 10) as libc::c_schar;
            } else {
                carry = 0;
                *((*pA).a).offset(i as isize) = x as libc::c_schar;
            }
            i -= 1;
        }
    } else {
        let mut aA: *mut libc::c_schar = 0 as *mut libc::c_schar;
        let mut aB: *mut libc::c_schar = 0 as *mut libc::c_schar;
        let mut borrow: i32 = 0;
        rc = memcmp((*pA).a as *const libc::c_void, (*pB).a as *const libc::c_void, nDigit as u64);
        if rc < 0 {
            aA = (*pB).a;
            aB = (*pA).a;
            (*pA).sign = ((*pA).sign == 0) as i32 as i8;
        } else {
            aA = (*pA).a;
            aB = (*pB).a;
        }
        i = nDigit - 1;
        while i >= 0 {
            let mut x_0: i32 = *aA.offset(i as isize) as i32 - *aB.offset(i as isize) as i32 - borrow;
            if x_0 < 0 {
                *((*pA).a).offset(i as isize) = (x_0 + 10) as libc::c_schar;
                borrow = 1;
            } else {
                *((*pA).a).offset(i as isize) = x_0 as libc::c_schar;
                borrow = 0;
            }
            i -= 1;
        }
    };
}
unsafe extern "C" fn decimalCollFunc(
    mut notUsed: *mut libc::c_void,
    mut nKey1: i32,
    mut pKey1: *const libc::c_void,
    mut nKey2: i32,
    mut pKey2: *const libc::c_void,
) -> i32 {
    let mut zA: *const u8 = pKey1 as *const u8;
    let mut zB: *const u8 = pKey2 as *const u8;
    let mut pA: *mut Decimal = decimal_new(0 as *mut sqlite3_context, 0 as *mut sqlite3_value, nKey1, zA);
    let mut pB: *mut Decimal = decimal_new(0 as *mut sqlite3_context, 0 as *mut sqlite3_value, nKey2, zB);
    let mut rc: i32 = 0;
    if pA.is_null() || pB.is_null() {
        rc = 0;
    } else {
        rc = decimal_cmp(pA, pB);
    }
    decimal_free(pA);
    decimal_free(pB);
    return rc;
}
unsafe extern "C" fn decimalAddFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut pA: *mut Decimal = decimal_new(context, *argv.offset(0), 0, std::ptr::null());
    let mut pB: *mut Decimal = decimal_new(context, *argv.offset(1), 0, std::ptr::null());
    decimal_add(pA, pB);
    decimal_result(context, pA);
    decimal_free(pA);
    decimal_free(pB);
}
unsafe extern "C" fn decimalSubFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut pA: *mut Decimal = decimal_new(context, *argv.offset(0), 0, std::ptr::null());
    let mut pB: *mut Decimal = decimal_new(context, *argv.offset(1), 0, std::ptr::null());
    if !pB.is_null() {
        (*pB).sign = ((*pB).sign == 0) as i32 as i8;
        decimal_add(pA, pB);
        decimal_result(context, pA);
    }
    decimal_free(pA);
    decimal_free(pB);
}
unsafe extern "C" fn decimalSumStep(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut p: *mut Decimal = 0 as *mut Decimal;
    let mut pArg: *mut Decimal = 0 as *mut Decimal;
    p = sqlite3_aggregate_context(context, ::std::mem::size_of::<Decimal>() as u64 as i32) as *mut Decimal;
    if p.is_null() {
        return;
    }
    if (*p).isInit == 0 {
        (*p).isInit = 1 as i8;
        (*p).a = sqlite3_malloc(2) as *mut libc::c_schar;
        if ((*p).a).is_null() {
            (*p).oom = 1 as i8;
        } else {
            *((*p).a).offset(0) = 0 as libc::c_schar;
        }
        (*p).nDigit = 1;
        (*p).nFrac = 0;
    }
    if sqlite3_value_type(*argv.offset(0)) == 5 {
        return;
    }
    pArg = decimal_new(context, *argv.offset(0), 0, std::ptr::null());
    decimal_add(p, pArg);
    decimal_free(pArg);
}
unsafe extern "C" fn decimalSumInverse(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut p: *mut Decimal = 0 as *mut Decimal;
    let mut pArg: *mut Decimal = 0 as *mut Decimal;
    p = sqlite3_aggregate_context(context, ::std::mem::size_of::<Decimal>() as u64 as i32) as *mut Decimal;
    if p.is_null() {
        return;
    }
    if sqlite3_value_type(*argv.offset(0)) == 5 {
        return;
    }
    pArg = decimal_new(context, *argv.offset(0), 0, std::ptr::null());
    if !pArg.is_null() {
        (*pArg).sign = ((*pArg).sign == 0) as i32 as i8;
    }
    decimal_add(p, pArg);
    decimal_free(pArg);
}
unsafe extern "C" fn decimalSumValue(mut context: *mut sqlite3_context) {
    let mut p: *mut Decimal = sqlite3_aggregate_context(context, 0) as *mut Decimal;
    if p.is_null() {
        return;
    }
    decimal_result(context, p);
}
unsafe extern "C" fn decimalSumFinalize(mut context: *mut sqlite3_context) {
    let mut p: *mut Decimal = sqlite3_aggregate_context(context, 0) as *mut Decimal;
    if p.is_null() {
        return;
    }
    decimal_result(context, p);
    decimal_clear(p);
}
unsafe extern "C" fn decimalMulFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut pA: *mut Decimal = decimal_new(context, *argv.offset(0), 0, std::ptr::null());
    let mut pB: *mut Decimal = decimal_new(context, *argv.offset(1), 0, std::ptr::null());
    let mut acc: *mut libc::c_schar = 0 as *mut libc::c_schar;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    let mut minFrac: i32 = 0;
    if !(pA.is_null()
        || (*pA).oom as i32 != 0
        || (*pA).isNull as i32 != 0
        || pB.is_null()
        || (*pB).oom as i32 != 0
        || (*pB).isNull as i32 != 0)
    {
        acc = sqlite3_malloc64(((*pA).nDigit + (*pB).nDigit + 2) as u64) as *mut libc::c_schar;
        if acc.is_null() {
            sqlite3_result_error_nomem(context);
        } else {
            memset(acc as *mut libc::c_void, 0, ((*pA).nDigit + (*pB).nDigit + 2) as u64);
            minFrac = (*pA).nFrac;
            if (*pB).nFrac < minFrac {
                minFrac = (*pB).nFrac;
            }
            i = (*pA).nDigit - 1;
            while i >= 0 {
                let mut f: libc::c_schar = *((*pA).a).offset(i as isize);
                let mut carry: i32 = 0;
                let mut x: i32 = 0;
                j = (*pB).nDigit - 1;
                k = i + j + 3;
                while j >= 0 {
                    x = *acc.offset(k as isize) as i32 + f as i32 * *((*pB).a).offset(j as isize) as i32 + carry;
                    *acc.offset(k as isize) = (x % 10) as libc::c_schar;
                    carry = x / 10;
                    j -= 1;
                    k -= 1;
                }
                x = *acc.offset(k as isize) as i32 + carry;
                *acc.offset(k as isize) = (x % 10) as libc::c_schar;
                *acc.offset((k - 1) as isize) = (*acc.offset((k - 1) as isize) as i32 + x / 10) as libc::c_schar;
                i -= 1;
            }
            sqlite3_free((*pA).a as *mut libc::c_void);
            (*pA).a = acc;
            acc = 0 as *mut libc::c_schar;
            (*pA).nDigit += (*pB).nDigit + 2;
            (*pA).nFrac += (*pB).nFrac;
            (*pA).sign = ((*pA).sign as i32 ^ (*pB).sign as i32) as i8;
            while (*pA).nFrac > minFrac && *((*pA).a).offset(((*pA).nDigit - 1) as isize) as i32 == 0 {
                (*pA).nFrac -= 1;
                (*pA).nDigit -= 1;
            }
            decimal_result(context, pA);
        }
    }
    sqlite3_free(acc as *mut libc::c_void);
    decimal_free(pA);
    decimal_free(pB);
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_decimal_init(
    mut db: *mut sqlite3,
    mut pzErrMsg: *mut *mut i8,
    mut pApi: *const sqlite3_api_routines,
) -> i32 {
    const aFunc: [C2RustUnnamed_16; 5] = [
        C2RustUnnamed_16 {
            zFuncName: b"decimal\0" as *const u8 as *const i8,
            nArg: 1,
            xFunc: Some(decimalFunc),
        },
        C2RustUnnamed_16 {
            zFuncName: b"decimal_cmp\0" as *const u8 as *const i8,
            nArg: 2,
            xFunc: Some(decimalCmpFunc),
        },
        C2RustUnnamed_16 {
            zFuncName: b"decimal_add\0" as *const u8 as *const i8,
            nArg: 2,
            xFunc: Some(decimalAddFunc),
        },
        C2RustUnnamed_16 {
            zFuncName: b"decimal_sub\0" as *const u8 as *const i8,
            nArg: 2,
            xFunc: Some(decimalSubFunc),
        },
        C2RustUnnamed_16 {
            zFuncName: b"decimal_mul\0" as *const u8 as *const i8,
            nArg: 2,
            xFunc: Some(decimalMulFunc),
        },
    ];

    let mut rc: i32 = 0;
    for func in aFunc {
        if rc != 0 {
            break;
        }
        rc = sqlite3_create_function(
            db,
            func.zFuncName,
            func.nArg,
            1 | 0x200000 | 0x800,
            std::ptr::null_mut(),
            func.xFunc,
            None,
            None,
        );
    }

    if rc == 0 {
        rc = sqlite3_create_window_function(
            db,
            b"decimal_sum\0" as *const u8 as *const i8,
            1,
            1 | 0x200000 | 0x800,
            std::ptr::null_mut(),
            Some(decimalSumStep),
            Some(decimalSumFinalize),
            Some(decimalSumValue),
            Some(decimalSumInverse),
            None,
        );
    }
    if rc == 0 {
        rc = sqlite3_create_collation(
            db,
            b"decimal\0" as *const u8 as *const i8,
            1,
            std::ptr::null_mut(),
            Some(decimalCollFunc),
        );
    }

    return rc;
}

unsafe extern "C" fn ieee754func(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    if argc == 1 {
        let mut m: i64 = 0;
        let mut a: i64 = 0;
        let mut r: f64 = 0.;
        let mut e: i32 = 0;
        let mut isNeg: i32 = 0;
        let mut zResult: [i8; 100] = [0; 100];
        assert!(::std::mem::size_of::<i64>() as u64 == ::std::mem::size_of::<f64>() as u64);
        if sqlite3_value_type(*argv.offset(0)) == 4 && sqlite3_value_bytes(*argv.offset(0)) as u64 == ::std::mem::size_of::<f64>() as u64 {
            let mut x: *const u8 = sqlite3_value_blob(*argv.offset(0)) as *const u8;
            let mut v: u64 = 0;
            for i in 0..::std::mem::size_of::<f64>() {
                v = v << 8 | *x.offset(i as isize) as u64;
            }
            memcpy(
                &mut r as *mut f64 as *mut libc::c_void,
                &mut v as *mut u64 as *const libc::c_void,
                ::std::mem::size_of::<f64>() as u64,
            );
        } else {
            r = sqlite3_value_double(*argv.offset(0));
        }
        if r < 0.0f64 {
            isNeg = 1;
            r = -r;
        } else {
            isNeg = 0;
        }
        memcpy(
            &mut a as *mut i64 as *mut libc::c_void,
            &mut r as *mut f64 as *const libc::c_void,
            ::std::mem::size_of::<i64>() as u64,
        );
        if a == 0 {
            e = 0;
            m = 0;
        } else {
            e = (a >> 52) as i32;
            m = a & ((1 as i64) << 52) - 1 as i64;
            if e == 0 {
                m <<= 1;
            } else {
                m |= (1 as i64) << 52;
            }
            while e < 1075 && m > 0 && m & 1 as i64 == 0 {
                m >>= 1;
                e += 1;
            }
            if isNeg != 0 {
                m = -m;
            }
        }
        match *(sqlite3_user_data(context) as *mut i32) {
            0 => {
                sqlite3_snprintf(
                    ::std::mem::size_of::<[i8; 100]>() as u64 as i32,
                    zResult.as_mut_ptr(),
                    b"ieee754(%lld,%d)\0" as *const u8 as *const i8,
                    m,
                    e - 1075,
                );
                sqlite3_result_text(
                    context,
                    zResult.as_mut_ptr(),
                    -1,
                    ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
                );
            }
            1 => {
                sqlite3_result_int64(context, m);
            }
            2 => {
                sqlite3_result_int(context, e - 1075);
            }
            _ => {}
        }
    } else {
        let mut m_0: i64 = 0;
        let mut e_0: i64 = 0;
        let mut a_0: i64 = 0;
        let mut r_0: f64 = 0.;
        let mut isNeg_0: i32 = 0;
        m_0 = sqlite3_value_int64(*argv.offset(0));
        e_0 = sqlite3_value_int64(*argv.offset(1));
        if e_0 > 10000 as i64 {
            e_0 = 10000 as i64;
        } else if e_0 < -(10000) as i64 {
            e_0 = -(10000) as i64;
        }
        if m_0 < 0 {
            isNeg_0 = 1;
            m_0 = -m_0;
            if m_0 < 0 {
                return;
            }
        } else if m_0 == 0 && e_0 > -(1000) as i64 && e_0 < 1000 as i64 {
            sqlite3_result_double(context, 0.0f64);
            return;
        }
        while m_0 >> 32 & 0xffe00000 as i64 != 0 {
            m_0 >>= 1;
            e_0 += 1;
        }
        while m_0 != 0 && m_0 >> 32 & 0xfff00000 as i64 == 0 {
            m_0 <<= 1;
            e_0 -= 1;
        }
        e_0 += 1075 as i64;
        if e_0 <= 0 {
            if 1 as i64 - e_0 >= 64 as i32 as i64 {
                m_0 = 0;
            } else {
                m_0 >>= 1 as i64 - e_0;
            }
            e_0 = 0;
        } else if e_0 > 0x7ff as i32 as i64 {
            e_0 = 0x7ff as i32 as i64;
        }
        a_0 = m_0 & ((1 as i64) << 52) - 1 as i64;
        a_0 |= e_0 << 52;
        if isNeg_0 != 0 {
            a_0 = (a_0 as u64 | (1 as u64) << 63) as i64;
        }
        memcpy(
            &mut r_0 as *mut f64 as *mut libc::c_void,
            &mut a_0 as *mut i64 as *const libc::c_void,
            ::std::mem::size_of::<f64>() as u64,
        );
        sqlite3_result_double(context, r_0);
    };
}
unsafe extern "C" fn ieee754func_from_blob(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    if sqlite3_value_type(*argv.offset(0)) == 4 && sqlite3_value_bytes(*argv.offset(0)) as u64 == ::std::mem::size_of::<f64>() as u64 {
        let mut r: f64 = 0.;
        let mut x: *const u8 = sqlite3_value_blob(*argv.offset(0)) as *const u8;
        let mut v: u64 = 0;
        for i in 0..::std::mem::size_of::<f64>() {
            v = v << 8 | *x.offset(i as isize) as u64;
        }
        memcpy(
            &mut r as *mut f64 as *mut libc::c_void,
            &mut v as *mut u64 as *const libc::c_void,
            ::std::mem::size_of::<f64>() as u64,
        );
        sqlite3_result_double(context, r);
    }
}
unsafe extern "C" fn ieee754func_to_blob(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    if sqlite3_value_type(*argv.offset(0)) == 2 || sqlite3_value_type(*argv.offset(0)) == 1 {
        let mut r: f64 = sqlite3_value_double(*argv.offset(0));
        let mut v: u64 = 0;
        let mut a: [u8; 8] = [0; 8];
        memcpy(
            &mut v as *mut u64 as *mut libc::c_void,
            &mut r as *mut f64 as *const libc::c_void,
            ::std::mem::size_of::<f64>() as u64,
        );
        for i in 1..=::std::mem::size_of::<f64>() as u64 {
            a[(::std::mem::size_of::<f64>() as u64).wrapping_sub(i as u64) as usize] = (v & 0xff as i32 as u64) as u8;
            v >>= 8;
        }
        sqlite3_result_blob(
            context,
            a.as_mut_ptr() as *const libc::c_void,
            ::std::mem::size_of::<f64>() as u64 as i32,
            ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_ieee_init(mut db: *mut sqlite3, mut pzErrMsg: *mut *mut i8, mut pApi: *const sqlite3_api_routines) -> i32 {
    const aFunc: [C2RustUnnamed_17; 6] = [
        C2RustUnnamed_17 {
            zFName: b"ieee754\0" as *const u8 as *const i8 as *mut i8,
            nArg: 1,
            iAux: 0,
            xFunc: Some(ieee754func),
        },
        C2RustUnnamed_17 {
            zFName: b"ieee754\0" as *const u8 as *const i8 as *mut i8,
            nArg: 2,
            iAux: 0,
            xFunc: Some(ieee754func),
        },
        C2RustUnnamed_17 {
            zFName: b"ieee754_mantissa\0" as *const u8 as *const i8 as *mut i8,
            nArg: 1,
            iAux: 1,
            xFunc: Some(ieee754func),
        },
        C2RustUnnamed_17 {
            zFName: b"ieee754_exponent\0" as *const u8 as *const i8 as *mut i8,
            nArg: 1,
            iAux: 2,
            xFunc: Some(ieee754func),
        },
        C2RustUnnamed_17 {
            zFName: b"ieee754_to_blob\0" as *const u8 as *const i8 as *mut i8,
            nArg: 1,
            iAux: 0,
            xFunc: Some(ieee754func_to_blob),
        },
        C2RustUnnamed_17 {
            zFName: b"ieee754_from_blob\0" as *const u8 as *const i8 as *mut i8,
            nArg: 1,
            iAux: 0,
            xFunc: Some(ieee754func_from_blob),
        },
    ];
    let mut rc: i32 = 0;
    for func in aFunc {
        rc = sqlite3_create_function(
            db,
            func.zFName,
            func.nArg,
            1 | 0x200000,
            &func.iAux as *const i32 as *mut libc::c_void,
            func.xFunc,
            None,
            None,
        );
        if rc != 0 {
            break;
        }
    }
    return rc;
}
unsafe extern "C" fn seriesConnect(
    mut db: *mut sqlite3,
    mut pUnused: *mut libc::c_void,
    mut argcUnused: i32,
    mut argvUnused: *const *const i8,
    mut ppVtab: *mut *mut sqlite3_vtab,
    mut pzErrUnused: *mut *mut i8,
) -> i32 {
    let mut pNew: *mut sqlite3_vtab = std::ptr::null_mut();
    let mut rc: i32 = 0;
    rc = sqlite3_declare_vtab(
        db,
        b"CREATE TABLE x(value,start hidden,stop hidden,step hidden)\0" as *const u8 as *const i8,
    );
    if rc == 0 {
        *ppVtab = sqlite3_malloc(::std::mem::size_of::<sqlite3_vtab>() as u64 as i32) as *mut sqlite3_vtab;
        pNew = *ppVtab;
        if pNew.is_null() {
            return 7 as i32;
        }
        memset(pNew as *mut libc::c_void, 0, ::std::mem::size_of::<sqlite3_vtab>() as u64);
        sqlite3_vtab_config(db, 2);
    }
    return rc;
}
unsafe extern "C" fn seriesDisconnect(mut pVtab: *mut sqlite3_vtab) -> i32 {
    sqlite3_free(pVtab as *mut libc::c_void);
    return 0;
}
unsafe extern "C" fn seriesOpen(mut pUnused: *mut sqlite3_vtab, mut ppCursor: *mut *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut series_cursor = 0 as *mut series_cursor;
    pCur = sqlite3_malloc(::std::mem::size_of::<series_cursor>() as u64 as i32) as *mut series_cursor;
    if pCur.is_null() {
        return 7 as i32;
    }
    memset(pCur as *mut libc::c_void, 0, ::std::mem::size_of::<series_cursor>() as u64);
    *ppCursor = &mut (*pCur).base;
    return 0;
}
unsafe extern "C" fn seriesClose(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    sqlite3_free(cur as *mut libc::c_void);
    return 0;
}
unsafe extern "C" fn seriesNext(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut series_cursor = cur as *mut series_cursor;
    if (*pCur).isDesc != 0 {
        (*pCur).iValue -= (*pCur).iStep;
    } else {
        (*pCur).iValue += (*pCur).iStep;
    }
    (*pCur).iRowid += 1;
    return 0;
}
unsafe extern "C" fn seriesColumn(mut cur: *mut sqlite3_vtab_cursor, mut ctx: *mut sqlite3_context, mut i: i32) -> i32 {
    let mut pCur: *mut series_cursor = cur as *mut series_cursor;
    let x: i64 = match i {
        1 => (*pCur).mnValue,
        2 => (*pCur).mxValue,
        3 => (*pCur).iStep,
        _ => (*pCur).iValue,
    };
    sqlite3_result_int64(ctx, x);
    return 0;
}
unsafe extern "C" fn seriesRowid(mut cur: *mut sqlite3_vtab_cursor, mut pRowid: *mut i64) -> i32 {
    let pCur = cur as *mut series_cursor;
    *pRowid = (*pCur).iRowid;
    return 0;
}
unsafe extern "C" fn seriesEof(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCur: *mut series_cursor = cur as *mut series_cursor;
    if (*pCur).isDesc != 0 {
        return ((*pCur).iValue < (*pCur).mnValue) as i32;
    } else {
        return ((*pCur).iValue > (*pCur).mxValue) as i32;
    };
}
unsafe extern "C" fn seriesFilter(
    mut pVtabCursor: *mut sqlite3_vtab_cursor,
    mut idxNum: i32,
    mut idxStrUnused: *const i8,
    mut argc: i32,
    mut argv: *mut *mut sqlite3_value,
) -> i32 {
    let mut pCur: *mut series_cursor = pVtabCursor as *mut series_cursor;
    let mut i: i32 = 0;
    if idxNum & 1 != 0 {
        (*pCur).mnValue = sqlite3_value_int64(*argv.offset(i as isize));
        i += 1;
    } else {
        (*pCur).mnValue = 0;
    }
    if idxNum & 2 != 0 {
        (*pCur).mxValue = sqlite3_value_int64(*argv.offset(i as isize));
        i += 1;
    } else {
        (*pCur).mxValue = 0xffffffff as u32 as i64;
    }
    if idxNum & 4 != 0 {
        (*pCur).iStep = sqlite3_value_int64(*argv.offset(i as isize));
        i += 1;
        if (*pCur).iStep == 0 {
            (*pCur).iStep = 1 as i64;
        } else if (*pCur).iStep < 0 {
            (*pCur).iStep = -(*pCur).iStep;
            if idxNum & 16 == 0 {
                idxNum |= 8;
            }
        }
    } else {
        (*pCur).iStep = 1 as i64;
    }

    for i in 0..argc {
        if sqlite3_value_type(*argv.offset(i as isize)) == 5 {
            (*pCur).mnValue = 1 as i64;
            (*pCur).mxValue = 0;
            break;
        }
    }

    if idxNum & 8 != 0 {
        (*pCur).isDesc = 1;
        (*pCur).iValue = (*pCur).mxValue;
        if (*pCur).iStep > 0 {
            (*pCur).iValue -= ((*pCur).mxValue - (*pCur).mnValue) % (*pCur).iStep;
        }
    } else {
        (*pCur).isDesc = 0;
        (*pCur).iValue = (*pCur).mnValue;
    }
    (*pCur).iRowid = 1;
    return 0;
}
unsafe extern "C" fn seriesBestIndex(mut pVTab: *mut sqlite3_vtab, mut pIdxInfo: *mut sqlite3_index_info) -> i32 {
    let mut idxNum: i32 = 0;
    let mut bStartSeen: i32 = 0;
    let mut unusableMask: i32 = 0;
    let mut nArg: i32 = 0;
    let mut aIdx: [i32; 3] = [0; 3];
    let mut pConstraint: *const sqlite3_index_constraint = 0 as *const sqlite3_index_constraint;
    assert!(2 == 1 + 1);
    assert!(3 == 1 + 2);
    aIdx[2] = -1;
    aIdx[1] = aIdx[2];
    aIdx[0] = aIdx[1];
    pConstraint = (*pIdxInfo).aConstraint;

    for i in 0..(*pIdxInfo).nConstraint {
        if !((*pConstraint).iColumn < 1) {
            let iCol = (*pConstraint).iColumn - 1;
            assert!(iCol >= 0 && iCol <= 2);
            let iMask = (1) << iCol;
            if iCol == 0 {
                bStartSeen = 1;
            }
            if (*pConstraint).usable == 0 {
                unusableMask |= iMask;
            } else if (*pConstraint).op == 2 {
                idxNum |= iMask;
                aIdx[iCol as usize] = i;
            }
        }
        pConstraint = pConstraint.offset(1);
    }

    for i in 0..3 {
        let j = aIdx[i as usize];
        if j >= 0 {
            nArg += 1;
            (*((*pIdxInfo).aConstraintUsage).offset(j as isize)).argvIndex = nArg;
            (*((*pIdxInfo).aConstraintUsage).offset(j as isize)).omit = (0 == 0) as i32 as u8;
        }
    }
    if bStartSeen == 0 {
        sqlite3_free((*pVTab).zErrMsg as *mut libc::c_void);
        (*pVTab).zErrMsg = sqlite3_mprintf(b"first argument to \"generate_series()\" missing or unusable\0" as *const u8 as *const i8);
        return 1;
    }
    if unusableMask & !idxNum != 0 {
        return 19;
    }
    if idxNum & 3 == 3 {
        (*pIdxInfo).estimatedCost = (2 - (idxNum & 4 != 0) as i32) as f64;
        (*pIdxInfo).estimatedRows = 1000 as i64;
        if (*pIdxInfo).nOrderBy >= 1 && (*((*pIdxInfo).aOrderBy).offset(0)).iColumn == 0 {
            if (*((*pIdxInfo).aOrderBy).offset(0)).desc != 0 {
                idxNum |= 8;
            } else {
                idxNum |= 16;
            }
            (*pIdxInfo).orderByConsumed = 1;
        }
    } else {
        (*pIdxInfo).estimatedRows = 2147483647 as i32 as i64;
    }
    (*pIdxInfo).idxNum = idxNum;
    return 0;
}

static mut seriesModule: sqlite3_module = unsafe {
    sqlite3_module {
        iVersion: 0,
        xCreate: None,
        xConnect: Some(seriesConnect),
        xBestIndex: Some(seriesBestIndex),
        xDisconnect: Some(seriesDisconnect),
        xDestroy: None,
        xOpen: Some(seriesOpen),
        xClose: Some(seriesClose),
        xFilter: Some(seriesFilter),
        xNext: Some(seriesNext),
        xEof: Some(seriesEof),
        xColumn: Some(seriesColumn),
        xRowid: Some(seriesRowid),
        xUpdate: None,
        xBegin: None,
        xSync: None,
        xCommit: None,
        xRollback: None,
        xFindFunction: None,
        xRename: None,
        xSavepoint: None,
        xRelease: None,
        xRollbackTo: None,
        xShadowName: None,
    }
};

#[no_mangle]
pub unsafe extern "C" fn sqlite3_series_init(
    mut db: *mut sqlite3,
    mut pzErrMsg: *mut *mut i8,
    mut pApi: *const sqlite3_api_routines,
) -> i32 {
    let mut rc: i32 = 0;
    if sqlite3_libversion_number() < 3008012 && !pzErrMsg.is_null() {
        *pzErrMsg = sqlite3_mprintf(b"generate_series() requires SQLite 3.8.12 or later\0" as *const u8 as *const i8);
        return 1;
    }
    rc = sqlite3_create_module(
        db,
        b"generate_series\0" as *const u8 as *const i8,
        &mut seriesModule,
        std::ptr::null_mut(),
    );
    return rc;
}
unsafe extern "C" fn re_add_state(mut pSet: *mut ReStateSet, mut newState: i32) {
    for i in 0..(*pSet).nState {
        if *((*pSet).aState).offset(i as isize) as i32 == newState {
            return;
        }
    }
    *((*pSet).aState).offset((*pSet).nState as isize) = newState as ReStateNumber;
    (*pSet).nState += 1;
}
unsafe extern "C" fn re_next_char(mut p: *mut ReInput) -> u32 {
    let mut c: u32 = 0;
    if (*p).i >= (*p).mx {
        return 0;
    }
    c = *((*p).z).offset((*p).i as isize) as u32;
    (*p).i += 1;
    if c >= 0x80 {
        if c & 0xe0 == 0xc0 && (*p).i < (*p).mx && *((*p).z).offset((*p).i as isize) as i32 & 0xc0 == 0x80 {
            c = (c & 0x1f as i32 as u32) << 6 | (*((*p).z).offset((*p).i as isize) as i32 & 0x3f as i32) as u32;
            (*p).i += 1;
            if c < 0x80 {
                c = 0xfffd as i32 as u32;
            }
        } else if c & 0xf0 == 0xe0
            && ((*p).i + 1) < (*p).mx
            && *((*p).z).offset((*p).i as isize) as i32 & 0xc0 == 0x80
            && *((*p).z).offset(((*p).i + 1) as isize) as i32 & 0xc0 == 0x80
        {
            c = (c & 0xf as i32 as u32) << 12
                | ((*((*p).z).offset((*p).i as isize) as i32 & 0x3f as i32) << 6) as u32
                | (*((*p).z).offset(((*p).i + 1) as isize) as i32 & 0x3f as i32) as u32;
            (*p).i += 2;
            if c <= 0x7ff as i32 as u32 || c >= 0xd800 && c <= 0xdfff as i32 as u32 {
                c = 0xfffd as i32 as u32;
            }
        } else if c & 0xf8 == 0xf0
            && ((*p).i + 3) < (*p).mx
            && *((*p).z).offset((*p).i as isize) as i32 & 0xc0 == 0x80
            && *((*p).z).offset(((*p).i + 1) as isize) as i32 & 0xc0 == 0x80
            && *((*p).z).offset(((*p).i + 2) as isize) as i32 & 0xc0 == 0x80
        {
            c = (c & 0x7 as i32 as u32) << 18
                | ((*((*p).z).offset((*p).i as isize) as i32 & 0x3f as i32) << 12) as u32
                | ((*((*p).z).offset(((*p).i + 1) as isize) as i32 & 0x3f as i32) << 6) as u32
                | (*((*p).z).offset(((*p).i + 2) as isize) as i32 & 0x3f as i32) as u32;
            (*p).i += 3;
            if c <= 0xffff as i32 as u32 || c > 0x10ffff as i32 as u32 {
                c = 0xfffd as i32 as u32;
            }
        } else {
            c = 0xfffd as i32 as u32;
        }
    }
    return c;
}
unsafe extern "C" fn re_next_char_nocase(mut p: *mut ReInput) -> u32 {
    let mut c: u32 = re_next_char(p);
    if c >= 'A' as i32 as u32 && c <= 'Z' as i32 as u32 {
        c = c.wrapping_add(('a' as i32 - 'A' as i32) as u32);
    }
    return c;
}
unsafe extern "C" fn re_word_char(mut c: i32) -> i32 {
    return (c >= '0' as i32 && c <= '9' as i32
        || c >= 'a' as i32 && c <= 'z' as i32
        || c >= 'A' as i32 && c <= 'Z' as i32
        || c == '_' as i32) as i32;
}
unsafe extern "C" fn re_digit_char(mut c: i32) -> i32 {
    return (c >= '0' as i32 && c <= '9' as i32) as i32;
}
unsafe extern "C" fn re_space_char(mut c: i32) -> i32 {
    return (c == ' ' as i32 || c == '\t' as i32 || c == '\n' as i32 || c == '\r' as i32 || c == '\u{b}' as i32 || c == '\u{c}' as i32)
        as i32;
}
unsafe extern "C" fn sqlite3re_match(mut pRe: *mut ReCompiled, mut zIn: *const u8, mut nIn: i32) -> i32 {
    let mut current_block: u64;
    let mut aStateSet: [ReStateSet; 2] = [ReStateSet {
        nState: 0,
        aState: 0 as *mut ReStateNumber,
    }; 2];
    let mut pThis: *mut ReStateSet = 0 as *mut ReStateSet;
    let mut pNext: *mut ReStateSet = 0 as *mut ReStateSet;
    let mut aSpace: [ReStateNumber; 100] = [0; 100];
    let mut pToFree: *mut ReStateNumber = 0 as *mut ReStateNumber;
    let mut iSwap: u32 = 0;
    let mut c: i32 = 0 + 1;
    let mut cPrev: i32 = 0;
    let mut rc: i32 = 0;
    let mut in_0: ReInput = ReInput {
        z: std::ptr::null(),
        i: 0,
        mx: 0,
    };
    in_0.z = zIn;
    in_0.i = 0;
    in_0.mx = if nIn >= 0 { nIn } else { strlen(zIn as *const i8) as i32 };
    if (*pRe).nInit != 0 {
        let mut x: u8 = (*pRe).zInit[0];
        while in_0.i + (*pRe).nInit <= in_0.mx
            && (*zIn.offset(in_0.i as isize) as i32 != x as i32
                || strncmp(
                    (zIn as *const i8).offset(in_0.i as isize),
                    ((*pRe).zInit).as_mut_ptr() as *const i8,
                    (*pRe).nInit as u64,
                ) != 0)
        {
            in_0.i += 1;
        }
        if in_0.i + (*pRe).nInit > in_0.mx {
            return 0;
        }
    }
    if (*pRe).nState as u64
        <= (::std::mem::size_of::<[ReStateNumber; 100]>() as u64)
            .wrapping_div((::std::mem::size_of::<ReStateNumber>() as u64).wrapping_mul(2 as u64))
    {
        pToFree = 0 as *mut ReStateNumber;
        aStateSet[0].aState = aSpace.as_mut_ptr();
    } else {
        pToFree = sqlite3_malloc64(
            (::std::mem::size_of::<ReStateNumber>() as u64)
                .wrapping_mul(2 as u64)
                .wrapping_mul((*pRe).nState as u64) as u64,
        ) as *mut ReStateNumber;
        if pToFree.is_null() {
            return -1;
        }
        aStateSet[0].aState = pToFree;
    }
    aStateSet[1].aState = &mut *((*aStateSet.as_mut_ptr().offset(0)).aState).offset((*pRe).nState as isize) as *mut ReStateNumber;
    pNext = &mut *aStateSet.as_mut_ptr().offset(1) as *mut ReStateSet;
    (*pNext).nState = 0;
    re_add_state(pNext, 0);
    's_132: loop {
        if !(c != 0 && (*pNext).nState > 0) {
            current_block = 14865402277128115059;
            break;
        }
        cPrev = c;
        c = ((*pRe).xNextChar).expect("non-null function pointer")(&mut in_0) as i32;
        pThis = pNext;
        pNext = &mut *aStateSet.as_mut_ptr().offset(iSwap as isize) as *mut ReStateSet;
        iSwap = 1 - iSwap;
        (*pNext).nState = 0;
        for i in 0..(*pThis).nState {
            let mut x_0: i32 = *((*pThis).aState).offset(i as isize) as i32;
            match *((*pRe).aOp).offset(x_0 as isize) as i32 {
                1 => {
                    if *((*pRe).aArg).offset(x_0 as isize) == c {
                        re_add_state(pNext, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                2 => {
                    if c != 0 {
                        re_add_state(pNext, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                11 => {
                    if re_word_char(c) != 0 {
                        re_add_state(pNext, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                12 => {
                    if re_word_char(c) == 0 && c != 0 {
                        re_add_state(pNext, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                13 => {
                    if re_digit_char(c) != 0 {
                        re_add_state(pNext, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                14 => {
                    if re_digit_char(c) == 0 && c != 0 {
                        re_add_state(pNext, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                15 => {
                    if re_space_char(c) != 0 {
                        re_add_state(pNext, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                16 => {
                    if re_space_char(c) == 0 && c != 0 {
                        re_add_state(pNext, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                17 => {
                    if re_word_char(c) != re_word_char(cPrev) {
                        re_add_state(pThis, x_0 + 1);
                    }
                    current_block = 18377268871191777778;
                }
                3 => {
                    re_add_state(pNext, x_0);
                    re_add_state(pThis, x_0 + 1);
                    current_block = 18377268871191777778;
                }
                4 => {
                    re_add_state(pThis, x_0 + *((*pRe).aArg).offset(x_0 as isize));
                    re_add_state(pThis, x_0 + 1);
                    current_block = 18377268871191777778;
                }
                5 => {
                    re_add_state(pThis, x_0 + *((*pRe).aArg).offset(x_0 as isize));
                    current_block = 18377268871191777778;
                }
                6 => {
                    rc = 1;
                    current_block = 4159726692890947844;
                    break 's_132;
                }
                8 => {
                    if c == 0 {
                        current_block = 18377268871191777778;
                    } else {
                        current_block = 17239133558811367971;
                    }
                }
                7 => {
                    current_block = 17239133558811367971;
                }
                _ => {
                    current_block = 18377268871191777778;
                }
            }
            match current_block {
                17239133558811367971 => {
                    let mut j: i32 = 1;
                    let mut n: i32 = *((*pRe).aArg).offset(x_0 as isize);
                    let mut hit: i32 = 0;
                    j = 1;
                    while j > 0 && j < n {
                        if *((*pRe).aOp).offset((x_0 + j) as isize) as i32 == 9 {
                            if *((*pRe).aArg).offset((x_0 + j) as isize) == c {
                                hit = 1;
                                j = -1;
                            }
                        } else if *((*pRe).aArg).offset((x_0 + j) as isize) <= c && *((*pRe).aArg).offset((x_0 + j + 1) as isize) >= c {
                            hit = 1;
                            j = -1;
                        } else {
                            j += 1;
                        }
                        j += 1;
                    }
                    if *((*pRe).aOp).offset(x_0 as isize) as i32 == 8 {
                        hit = (hit == 0) as i32;
                    }
                    if hit != 0 {
                        re_add_state(pNext, x_0 + n);
                    }
                }
                _ => {}
            }
        }
    }
    match current_block {
        14865402277128115059 => {
            for i in 0..(*pNext).nState {
                if *((*pRe).aOp).offset(*((*pNext).aState).offset(i as isize) as isize) as i32 == 6 {
                    rc = 1;
                    break;
                }
            }
        }
        _ => {}
    }
    sqlite3_free(pToFree as *mut libc::c_void);
    return rc;
}
unsafe extern "C" fn re_resize(mut p: *mut ReCompiled, mut N: i32) -> i32 {
    let mut aOp: *mut i8 = std::ptr::null_mut();
    let mut aArg: *mut i32 = 0 as *mut i32;
    aOp = sqlite3_realloc64(
        (*p).aOp as *mut libc::c_void,
        (N as u64).wrapping_mul(::std::mem::size_of::<i8>() as u64) as u64,
    ) as *mut i8;
    if aOp.is_null() {
        return 1;
    }
    (*p).aOp = aOp;
    aArg = sqlite3_realloc64(
        (*p).aArg as *mut libc::c_void,
        (N as u64).wrapping_mul(::std::mem::size_of::<i32>() as u64) as u64,
    ) as *mut i32;
    if aArg.is_null() {
        return 1;
    }
    (*p).aArg = aArg;
    (*p).nAlloc = N as u32;
    return 0;
}
unsafe extern "C" fn re_insert(mut p: *mut ReCompiled, mut iBefore: i32, mut op: i32, mut arg: i32) -> i32 {
    let mut i: i32 = 0;
    if (*p).nAlloc <= (*p).nState && re_resize(p, ((*p).nAlloc).wrapping_mul(2 as u32) as i32) != 0 {
        return 0;
    }
    i = (*p).nState as i32;
    while i > iBefore {
        *((*p).aOp).offset(i as isize) = *((*p).aOp).offset((i - 1) as isize);
        *((*p).aArg).offset(i as isize) = *((*p).aArg).offset((i - 1) as isize);
        i -= 1;
    }
    (*p).nState += 1;
    *((*p).aOp).offset(iBefore as isize) = op as i8;
    *((*p).aArg).offset(iBefore as isize) = arg;
    return iBefore;
}
unsafe extern "C" fn re_append(mut p: *mut ReCompiled, mut op: i32, mut arg: i32) -> i32 {
    return re_insert(p, (*p).nState as i32, op, arg);
}
unsafe extern "C" fn re_copy(mut p: *mut ReCompiled, mut iStart: i32, mut N: i32) {
    if ((*p).nState).wrapping_add(N as u32) >= (*p).nAlloc
        && re_resize(p, ((*p).nAlloc).wrapping_mul(2 as u32).wrapping_add(N as u32) as i32) != 0
    {
        return;
    }
    memcpy(
        &mut *((*p).aOp).offset((*p).nState as isize) as *mut i8 as *mut libc::c_void,
        &mut *((*p).aOp).offset(iStart as isize) as *mut i8 as *const libc::c_void,
        (N as u64).wrapping_mul(::std::mem::size_of::<i8>() as u64),
    );
    memcpy(
        &mut *((*p).aArg).offset((*p).nState as isize) as *mut i32 as *mut libc::c_void,
        &mut *((*p).aArg).offset(iStart as isize) as *mut i32 as *const libc::c_void,
        (N as u64).wrapping_mul(::std::mem::size_of::<i32>() as u64),
    );
    (*p).nState += N as u32;
}

unsafe extern "C" fn re_hex(mut c: i32, mut pV: *mut i32) -> i32 {
    if c >= '0' as i32 && c <= '9' as i32 {
        c -= '0' as i32;
    } else if c >= 'a' as i32 && c <= 'f' as i32 {
        c -= 'a' as i32 - 10;
    } else if c >= 'A' as i32 && c <= 'F' as i32 {
        c -= 'A' as i32 - 10;
    } else {
        return 0;
    }
    *pV = *pV * 16 + (c & 0xff as i32);
    return 1;
}
unsafe extern "C" fn re_esc_char(mut p: *mut ReCompiled) -> u32 {
    static mut zEsc: [i8; 21] = unsafe { *::std::mem::transmute::<&[u8; 21], &[i8; 21]>(b"afnrtv\\()*.+?[$^{|}]\0") };
    static mut zTrans: [i8; 7] = unsafe { *::std::mem::transmute::<&[u8; 7], &[i8; 7]>(b"\x07\x0C\n\r\t\x0B\0") };
    let mut i: i32 = 0;
    let mut v: i32 = 0;
    let mut c: i8 = 0;
    if (*p).sIn.i >= (*p).sIn.mx {
        return 0;
    }
    c = *((*p).sIn.z).offset((*p).sIn.i as isize) as i8;
    if c as i32 == 'u' as i32 && ((*p).sIn.i + 4) < (*p).sIn.mx {
        let mut zIn: *const u8 = ((*p).sIn.z).offset((*p).sIn.i as isize);
        if re_hex(*zIn.offset(1) as i32, &mut v) != 0
            && re_hex(*zIn.offset(2 as isize) as i32, &mut v) != 0
            && re_hex(*zIn.offset(3 as isize) as i32, &mut v) != 0
            && re_hex(*zIn.offset(4 as isize) as i32, &mut v) != 0
        {
            (*p).sIn.i += 5;
            return v as u32;
        }
    }
    if c as i32 == 'x' as i32 && ((*p).sIn.i + 2) < (*p).sIn.mx {
        let mut zIn_0: *const u8 = ((*p).sIn.z).offset((*p).sIn.i as isize);
        if re_hex(*zIn_0.offset(1) as i32, &mut v) != 0 && re_hex(*zIn_0.offset(2 as isize) as i32, &mut v) != 0 {
            (*p).sIn.i += 3;
            return v as u32;
        }
    }
    i = 0;
    while zEsc[i as usize] as i32 != 0 && zEsc[i as usize] as i32 != c as i32 {
        i += 1;
    }
    if zEsc[i as usize] != 0 {
        if i < 6 {
            c = zTrans[i as usize];
        }
        (*p).sIn.i += 1;
    } else {
        (*p).zErr = b"unknown \\ escape\0" as *const u8 as *const i8;
    }
    return c as u32;
}
unsafe extern "C" fn rePeek(mut p: *mut ReCompiled) -> u8 {
    return (if (*p).sIn.i < (*p).sIn.mx {
        *((*p).sIn.z).offset((*p).sIn.i as isize) as i32
    } else {
        0
    }) as u8;
}
unsafe extern "C" fn re_subcompile_re(mut p: *mut ReCompiled) -> *const i8 {
    let mut zErr: *const i8 = 0 as *const i8;
    let mut iStart: i32 = 0;
    let mut iEnd: i32 = 0;
    let mut iGoto: i32 = 0;
    iStart = (*p).nState as i32;
    zErr = re_subcompile_string(p);
    if !zErr.is_null() {
        return zErr;
    }
    while rePeek(p) as i32 == '|' as i32 {
        iEnd = (*p).nState as i32;
        re_insert(p, iStart, 4, iEnd + 2 - iStart);
        iGoto = re_append(p, 5, 0);
        (*p).sIn.i += 1;
        zErr = re_subcompile_string(p);
        if !zErr.is_null() {
            return zErr;
        }
        *((*p).aArg).offset(iGoto as isize) = ((*p).nState).wrapping_sub(iGoto as u32) as i32;
    }
    return 0 as *const i8;
}
unsafe extern "C" fn re_subcompile_string(mut p: *mut ReCompiled) -> *const i8 {
    let mut iPrev: i32 = -1;
    let mut iStart: i32 = 0;
    let mut c: u32 = 0;
    let mut zErr: *const i8 = 0 as *const i8;
    loop {
        c = ((*p).xNextChar).expect("non-null function pointer")(&mut (*p).sIn);
        if !(c != 0) {
            break;
        }
        iStart = (*p).nState as i32;
        match c {
            124 | 36 | 41 => {
                (*p).sIn.i -= 1;
                return 0 as *const i8;
            }
            40 => {
                zErr = re_subcompile_re(p);
                if !zErr.is_null() {
                    return zErr;
                }
                if rePeek(p) as i32 != ')' as i32 {
                    return b"unmatched '('\0" as *const u8 as *const i8;
                }
                (*p).sIn.i += 1;
            }
            46 => {
                if rePeek(p) as i32 == '*' as i32 {
                    re_append(p, 3, 0);
                    (*p).sIn.i += 1;
                } else {
                    re_append(p, 2, 0);
                }
            }
            42 => {
                if iPrev < 0 {
                    return b"'*' without operand\0" as *const u8 as *const i8;
                }
                re_insert(p, iPrev, 5, ((*p).nState).wrapping_sub(iPrev as u32).wrapping_add(1) as i32);
                re_append(p, 4, (iPrev as u32).wrapping_sub((*p).nState).wrapping_add(1) as i32);
            }
            43 => {
                if iPrev < 0 {
                    return b"'+' without operand\0" as *const u8 as *const i8;
                }
                re_append(p, 4, (iPrev as u32).wrapping_sub((*p).nState) as i32);
            }
            63 => {
                if iPrev < 0 {
                    return b"'?' without operand\0" as *const u8 as *const i8;
                }
                re_insert(p, iPrev, 4, ((*p).nState).wrapping_sub(iPrev as u32).wrapping_add(1) as i32);
            }
            123 => {
                let mut m: i32 = 0;
                let mut n: i32 = 0;
                let mut sz: i32 = 0;
                if iPrev < 0 {
                    return b"'{m,n}' without operand\0" as *const u8 as *const i8;
                }
                loop {
                    c = rePeek(p) as u32;
                    if !(c >= '0' as i32 as u32 && c <= '9' as i32 as u32) {
                        break;
                    }
                    m = ((m * 10) as u32).wrapping_add(c).wrapping_sub('0' as i32 as u32) as i32;
                    (*p).sIn.i += 1;
                }
                n = m;
                if c == ',' as i32 as u32 {
                    (*p).sIn.i += 1;
                    n = 0;
                    loop {
                        c = rePeek(p) as u32;
                        if !(c >= '0' as i32 as u32 && c <= '9' as i32 as u32) {
                            break;
                        }
                        n = ((n * 10) as u32).wrapping_add(c).wrapping_sub('0' as i32 as u32) as i32;
                        (*p).sIn.i += 1;
                    }
                }
                if c != '}' as i32 as u32 {
                    return b"unmatched '{'\0" as *const u8 as *const i8;
                }
                if n > 0 && n < m {
                    return b"n less than m in '{m,n}'\0" as *const u8 as *const i8;
                }
                (*p).sIn.i += 1;
                sz = ((*p).nState).wrapping_sub(iPrev as u32) as i32;
                if m == 0 {
                    if n == 0 {
                        return b"both m and n are zero in '{m,n}'\0" as *const u8 as *const i8;
                    }
                    re_insert(p, iPrev, 4, sz + 1);
                    n -= 1;
                } else {
                    for j in 1..m {
                        re_copy(p, iPrev, sz);
                    }
                }
                for j in m..n {
                    re_append(p, 4, sz + 1);
                    re_copy(p, iPrev, sz);
                }
                if n == 0 && m > 0 {
                    re_append(p, 4, -sz);
                }
            }
            91 => {
                let mut iFirst: i32 = (*p).nState as i32;
                if rePeek(p) as i32 == '^' as i32 {
                    re_append(p, 8, 0);
                    (*p).sIn.i += 1;
                } else {
                    re_append(p, 7 as i32, 0);
                }
                loop {
                    c = ((*p).xNextChar).expect("non-null function pointer")(&mut (*p).sIn);
                    if !(c != 0) {
                        break;
                    }
                    if c == '[' as i32 as u32 && rePeek(p) as i32 == ':' as i32 {
                        return b"POSIX character classes not supported\0" as *const u8 as *const i8;
                    }
                    if c == '\\' as i32 as u32 {
                        c = re_esc_char(p);
                    }
                    if rePeek(p) as i32 == '-' as i32 {
                        re_append(p, 10, c as i32);
                        (*p).sIn.i += 1;
                        c = ((*p).xNextChar).expect("non-null function pointer")(&mut (*p).sIn);
                        if c == '\\' as i32 as u32 {
                            c = re_esc_char(p);
                        }
                        re_append(p, 10, c as i32);
                    } else {
                        re_append(p, 9, c as i32);
                    }
                    if !(rePeek(p) as i32 == ']' as i32) {
                        continue;
                    }
                    (*p).sIn.i += 1;
                    break;
                }
                if c == 0 {
                    return b"unclosed '['\0" as *const u8 as *const i8;
                }
                *((*p).aArg).offset(iFirst as isize) = ((*p).nState).wrapping_sub(iFirst as u32) as i32;
            }
            92 => {
                let mut specialOp: i32 = 0;
                match rePeek(p) as i32 {
                    98 => {
                        specialOp = 17;
                    }
                    100 => {
                        specialOp = 13;
                    }
                    68 => {
                        specialOp = 14 as i32;
                    }
                    115 => {
                        specialOp = 15;
                    }
                    83 => {
                        specialOp = 16;
                    }
                    119 => {
                        specialOp = 11;
                    }
                    87 => {
                        specialOp = 12;
                    }
                    _ => {}
                }
                if specialOp != 0 {
                    (*p).sIn.i += 1;
                    re_append(p, specialOp, 0);
                } else {
                    c = re_esc_char(p);
                    re_append(p, 1, c as i32);
                }
            }
            _ => {
                re_append(p, 1, c as i32);
            }
        }
        iPrev = iStart;
    }
    return 0 as *const i8;
}
unsafe extern "C" fn sqlite3re_free(mut pRe: *mut ReCompiled) {
    if !pRe.is_null() {
        sqlite3_free((*pRe).aOp as *mut libc::c_void);
        sqlite3_free((*pRe).aArg as *mut libc::c_void);
        sqlite3_free(pRe as *mut libc::c_void);
    }
}
unsafe extern "C" fn sqlite3re_compile(mut ppRe: *mut *mut ReCompiled, mut zIn: *const i8, mut noCase: i32) -> *const i8 {
    let mut pRe: *mut ReCompiled = 0 as *mut ReCompiled;
    let mut zErr: *const i8 = 0 as *const i8;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    *ppRe = 0 as *mut ReCompiled;
    pRe = sqlite3_malloc(::std::mem::size_of::<ReCompiled>() as u64 as i32) as *mut ReCompiled;
    if pRe.is_null() {
        return b"out of memory\0" as *const u8 as *const i8;
    }
    memset(pRe as *mut libc::c_void, 0, ::std::mem::size_of::<ReCompiled>() as u64);
    (*pRe).xNextChar = if noCase != 0 {
        Some(re_next_char_nocase)
    } else {
        Some(re_next_char)
    };
    if re_resize(pRe, 30) != 0 {
        sqlite3re_free(pRe);
        return b"out of memory\0" as *const u8 as *const i8;
    }
    if *zIn.offset(0) as i32 == '^' as i32 {
        zIn = zIn.offset(1);
    } else {
        re_append(pRe, 3, 0);
    }
    (*pRe).sIn.z = zIn as *mut u8;
    (*pRe).sIn.i = 0;
    (*pRe).sIn.mx = strlen(zIn) as i32;
    zErr = re_subcompile_re(pRe);
    if !zErr.is_null() {
        sqlite3re_free(pRe);
        return zErr;
    }
    if rePeek(pRe) as i32 == '$' as i32 && (*pRe).sIn.i + 1 >= (*pRe).sIn.mx {
        re_append(pRe, 1, 0);
        re_append(pRe, 6, 0);
        *ppRe = pRe;
    } else if (*pRe).sIn.i >= (*pRe).sIn.mx {
        re_append(pRe, 6, 0);
        *ppRe = pRe;
    } else {
        sqlite3re_free(pRe);
        return b"unrecognized character\0" as *const u8 as *const i8;
    }
    if *((*pRe).aOp).offset(0) as i32 == 3 && noCase == 0 {
        j = 0;
        i = 1;
        while j < ::std::mem::size_of::<[u8; 12]>() as u64 as i32 - 2 && *((*pRe).aOp).offset(i as isize) as i32 == 1 {
            let mut x: u32 = *((*pRe).aArg).offset(i as isize) as u32;
            if x <= 127 as i32 as u32 {
                (*pRe).zInit[j as usize] = x as u8;
                j += 1;
            } else if x <= 0xfff as i32 as u32 {
                (*pRe).zInit[j as usize] = (0xc0 | x >> 6) as u8;
                j += 1;
                (*pRe).zInit[j as usize] = (0x80 | x & 0x3f as i32 as u32) as u8;
                j += 1;
            } else {
                if !(x <= 0xffff as i32 as u32) {
                    break;
                }
                (*pRe).zInit[j as usize] = (0xd0 | x >> 12) as u8;
                j += 1;
                (*pRe).zInit[j as usize] = (0x80 | x >> 6 & 0x3f as i32 as u32) as u8;
                j += 1;
                (*pRe).zInit[j as usize] = (0x80 | x & 0x3f as i32 as u32) as u8;
                j += 1;
            }
            i += 1;
        }
        if j > 0 && (*pRe).zInit[(j - 1) as usize] as i32 == 0 {
            j -= 1;
        }
        (*pRe).nInit = j;
    }
    return (*pRe).zErr;
}
unsafe extern "C" fn re_sql_func(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut pRe: *mut ReCompiled = 0 as *mut ReCompiled;
    let mut zPattern: *const i8 = 0 as *const i8;
    let mut zStr: *const u8 = std::ptr::null();
    let mut zErr: *const i8 = 0 as *const i8;
    let mut setAux: i32 = 0;
    pRe = sqlite3_get_auxdata(context, 0) as *mut ReCompiled;
    if pRe.is_null() {
        zPattern = sqlite3_value_text(*argv.offset(0)) as *const i8;
        if zPattern.is_null() {
            return;
        }
        zErr = sqlite3re_compile(&mut pRe, zPattern, (sqlite3_user_data(context) != std::ptr::null_mut()) as i32);
        if !zErr.is_null() {
            sqlite3re_free(pRe);
            sqlite3_result_error(context, zErr, -1);
            return;
        }
        if pRe.is_null() {
            sqlite3_result_error_nomem(context);
            return;
        }
        setAux = 1;
    }
    zStr = sqlite3_value_text(*argv.offset(1));
    if !zStr.is_null() {
        sqlite3_result_int(context, sqlite3re_match(pRe, zStr, -1));
    }
    if setAux != 0 {
        sqlite3_set_auxdata(
            context,
            0,
            pRe as *mut libc::c_void,
            ::std::mem::transmute::<
                Option<unsafe extern "C" fn(*mut ReCompiled) -> ()>,
                Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
            >(Some(sqlite3re_free)),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_regexp_init(
    mut db: *mut sqlite3,
    mut pzErrMsg: *mut *mut i8,
    mut pApi: *const sqlite3_api_routines,
) -> i32 {
    let mut rc: i32 = 0;
    rc = sqlite3_create_function(
        db,
        b"regexp\0" as *const u8 as *const i8,
        2,
        1 | 0x200000 | 0x800,
        std::ptr::null_mut(),
        Some(re_sql_func),
        None,
        None,
    );
    if rc == 0 {
        rc = sqlite3_create_function(
            db,
            b"regexpi\0" as *const u8 as *const i8,
            2,
            1 | 0x200000 | 0x800,
            db as *mut libc::c_void,
            Some(re_sql_func),
            None,
            None,
        );
    }
    return rc;
}
unsafe extern "C" fn idxMalloc(mut pRc: *mut i32, mut nByte: i32) -> *mut libc::c_void {
    let mut pRet: *mut libc::c_void = std::ptr::null_mut();
    assert!(*pRc == 0);
    assert!(nByte > 0);
    pRet = sqlite3_malloc(nByte);
    if !pRet.is_null() {
        memset(pRet, 0, nByte as u64);
    } else {
        *pRc = 7 as i32;
    }
    return pRet;
}
unsafe extern "C" fn idxHashInit(mut pHash: *mut IdxHash) {
    memset(pHash as *mut libc::c_void, 0, ::std::mem::size_of::<IdxHash>() as u64);
}
unsafe extern "C" fn idxHashClear(mut pHash: *mut IdxHash) {
    for i in 0..1023 {
        let mut pEntry: *mut IdxHashEntry = 0 as *mut IdxHashEntry;
        let mut pNext: *mut IdxHashEntry = 0 as *mut IdxHashEntry;
        pEntry = (*pHash).aHash[i as usize];
        while !pEntry.is_null() {
            pNext = (*pEntry).pHashNext;
            sqlite3_free((*pEntry).zVal2 as *mut libc::c_void);
            sqlite3_free(pEntry as *mut libc::c_void);
            pEntry = pNext;
        }
    }
    memset(pHash as *mut libc::c_void, 0, ::std::mem::size_of::<IdxHash>() as u64);
}
unsafe extern "C" fn idxHashString(mut z: *const i8, mut n: i32) -> i32 {
    let mut ret: u32 = 0;
    for i in 0..n {
        ret = ret.wrapping_add((ret << 3).wrapping_add(*z.offset(i as isize) as u32));
    }
    return ret.wrapping_rem(1023) as i32;
}
unsafe extern "C" fn idxHashAdd(mut pRc: *mut i32, mut pHash: *mut IdxHash, mut zKey: *const i8, mut zVal: *const i8) -> i32 {
    let mut nKey: i32 = strlen(zKey) as i32;
    let mut iHash: i32 = idxHashString(zKey, nKey);
    let mut nVal: i32 = if !zVal.is_null() { strlen(zVal) as i32 } else { 0 };
    let mut pEntry: *mut IdxHashEntry = 0 as *mut IdxHashEntry;
    assert!(iHash >= 0);
    pEntry = (*pHash).aHash[iHash as usize];
    while !pEntry.is_null() {
        if strlen((*pEntry).zKey) as i32 == nKey
            && 0 == memcmp((*pEntry).zKey as *const libc::c_void, zKey as *const libc::c_void, nKey as u64)
        {
            return 1;
        }
        pEntry = (*pEntry).pHashNext;
    }
    pEntry = idxMalloc(
        pRc,
        (::std::mem::size_of::<IdxHashEntry>() as u64)
            .wrapping_add(nKey as u64)
            .wrapping_add(1 as u64)
            .wrapping_add(nVal as u64)
            .wrapping_add(1 as u64) as i32,
    ) as *mut IdxHashEntry;
    if !pEntry.is_null() {
        (*pEntry).zKey = &mut *pEntry.offset(1) as *mut IdxHashEntry as *mut i8;
        memcpy((*pEntry).zKey as *mut libc::c_void, zKey as *const libc::c_void, nKey as u64);
        if !zVal.is_null() {
            (*pEntry).zVal = &mut *((*pEntry).zKey).offset((nKey + 1) as isize) as *mut i8;
            memcpy((*pEntry).zVal as *mut libc::c_void, zVal as *const libc::c_void, nVal as u64);
        }
        (*pEntry).pHashNext = (*pHash).aHash[iHash as usize];
        (*pHash).aHash[iHash as usize] = pEntry;
        (*pEntry).pNext = (*pHash).pFirst;
        (*pHash).pFirst = pEntry;
    }
    return 0;
}
unsafe extern "C" fn idxHashFind(mut pHash: *mut IdxHash, mut zKey: *const i8, mut nKey: i32) -> *mut IdxHashEntry {
    let mut iHash: i32 = 0;
    let mut pEntry: *mut IdxHashEntry = 0 as *mut IdxHashEntry;
    if nKey < 0 {
        nKey = strlen(zKey) as i32;
    }
    iHash = idxHashString(zKey, nKey);
    assert!(iHash >= 0);
    pEntry = (*pHash).aHash[iHash as usize];
    while !pEntry.is_null() {
        if strlen((*pEntry).zKey) as i32 == nKey
            && 0 == memcmp((*pEntry).zKey as *const libc::c_void, zKey as *const libc::c_void, nKey as u64)
        {
            return pEntry;
        }
        pEntry = (*pEntry).pHashNext;
    }
    return 0 as *mut IdxHashEntry;
}
unsafe extern "C" fn idxHashSearch(mut pHash: *mut IdxHash, mut zKey: *const i8, mut nKey: i32) -> *const i8 {
    let mut pEntry: *mut IdxHashEntry = idxHashFind(pHash, zKey, nKey);
    if !pEntry.is_null() {
        return (*pEntry).zVal;
    }
    return 0 as *const i8;
}
unsafe extern "C" fn idxNewConstraint(mut pRc: *mut i32, mut zColl: *const i8) -> *mut IdxConstraint {
    let mut pNew: *mut IdxConstraint = 0 as *mut IdxConstraint;
    let mut nColl: i32 = strlen(zColl) as i32;
    assert!(*pRc == 0);
    pNew = idxMalloc(
        pRc,
        (::std::mem::size_of::<IdxConstraint>() as u64)
            .wrapping_mul(nColl as u64)
            .wrapping_add(1 as u64) as i32,
    ) as *mut IdxConstraint;
    if !pNew.is_null() {
        (*pNew).zColl = &mut *pNew.offset(1) as *mut IdxConstraint as *mut i8;
        memcpy((*pNew).zColl as *mut libc::c_void, zColl as *const libc::c_void, (nColl + 1) as u64);
    }
    return pNew;
}
unsafe extern "C" fn idxDatabaseError(mut db: *mut sqlite3, mut pzErrmsg: *mut *mut i8) {
    *pzErrmsg = sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, sqlite3_errmsg(db));
}
unsafe extern "C" fn idxPrepareStmt(
    mut db: *mut sqlite3,
    mut ppStmt: *mut *mut sqlite3_stmt,
    mut pzErrmsg: *mut *mut i8,
    mut zSql: *const i8,
) -> i32 {
    let mut rc: i32 = sqlite3_prepare_v2(db, zSql, -1, ppStmt, 0 as *mut *const i8);
    if rc != 0 {
        *ppStmt = 0 as *mut sqlite3_stmt;
        idxDatabaseError(db, pzErrmsg);
    }
    return rc;
}
unsafe extern "C" fn idxPrintfPrepareStmt(
    mut db: *mut sqlite3,
    mut ppStmt: *mut *mut sqlite3_stmt,
    mut pzErrmsg: *mut *mut i8,
    mut zFmt: *const i8,
    mut args: ...
) -> i32 {
    let mut ap: ::std::ffi::VaListImpl;
    let mut rc: i32 = 0;
    let mut zSql: *mut i8 = std::ptr::null_mut();
    ap = args.clone();
    zSql = sqlite3_vmprintf(zFmt, ap.as_va_list());
    if zSql.is_null() {
        rc = 7 as i32;
    } else {
        rc = idxPrepareStmt(db, ppStmt, pzErrmsg, zSql);
        sqlite3_free(zSql as *mut libc::c_void);
    }
    return rc;
}
unsafe extern "C" fn expertDequote(mut zIn: *const i8) -> *mut i8 {
    let mut n: i32 = strlen(zIn) as i32;
    let mut zRet: *mut i8 = sqlite3_malloc(n) as *mut i8;
    assert!(*zIn.offset(0) as i32 == '\'' as i32);
    assert!(*zIn.offset((n - 1) as isize) as i32 == '\'' as i32);
    if !zRet.is_null() {
        let mut iOut: i32 = 0;
        let mut iIn: i32 = 0;
        iIn = 1;
        while iIn < n - 1 {
            if *zIn.offset(iIn as isize) as i32 == '\'' as i32 {
                assert!(*zIn.offset((iIn + 1) as isize) as i32 == '\'' as i32);
                iIn += 1;
            }
            *zRet.offset(iOut as isize) = *zIn.offset(iIn as isize);
            iOut += 1;
            iIn += 1;
        }
        *zRet.offset(iOut as isize) = '\0' as i32 as i8;
    }
    return zRet;
}
unsafe extern "C" fn expertConnect(
    mut db: *mut sqlite3,
    mut pAux: *mut libc::c_void,
    mut argc: i32,
    mut argv: *const *const i8,
    mut ppVtab: *mut *mut sqlite3_vtab,
    mut pzErr: *mut *mut i8,
) -> i32 {
    let mut pExpert: *mut sqlite3expert = pAux as *mut sqlite3expert;
    let mut p: *mut ExpertVtab = 0 as *mut ExpertVtab;
    let mut rc: i32 = 0;
    if argc != 4 {
        *pzErr = sqlite3_mprintf(b"internal error!\0" as *const u8 as *const i8);
        rc = 1;
    } else {
        let mut zCreateTable: *mut i8 = expertDequote(*argv.offset(3 as isize));
        if !zCreateTable.is_null() {
            rc = sqlite3_declare_vtab(db, zCreateTable);
            if rc == 0 {
                p = idxMalloc(&mut rc, ::std::mem::size_of::<ExpertVtab>() as u64 as i32) as *mut ExpertVtab;
            }
            if rc == 0 {
                (*p).pExpert = pExpert;
                (*p).pTab = (*pExpert).pTable;
                assert!(sqlite3_stricmp((*(*p).pTab).zName, *argv.offset(2 as isize),) == 0);
            }
            sqlite3_free(zCreateTable as *mut libc::c_void);
        } else {
            rc = 7 as i32;
        }
    }
    *ppVtab = p as *mut sqlite3_vtab;
    return rc;
}
unsafe extern "C" fn expertDisconnect(mut pVtab: *mut sqlite3_vtab) -> i32 {
    let mut p: *mut ExpertVtab = pVtab as *mut ExpertVtab;
    sqlite3_free(p as *mut libc::c_void);
    return 0;
}
unsafe extern "C" fn expertBestIndex(mut pVtab: *mut sqlite3_vtab, mut pIdxInfo: *mut sqlite3_index_info) -> i32 {
    let mut p: *mut ExpertVtab = pVtab as *mut ExpertVtab;
    let mut rc: i32 = 0;
    let mut n: i32 = 0;
    let mut pScan: *mut IdxScan = 0 as *mut IdxScan;
    let opmask: i32 = 2 | 4 | 16 | 32 | 8;
    pScan = idxMalloc(&mut rc, ::std::mem::size_of::<IdxScan>() as u64 as i32) as *mut IdxScan;
    if !pScan.is_null() {
        (*pScan).pTab = (*p).pTab;
        (*pScan).pNextScan = (*(*p).pExpert).pScan;
        (*(*p).pExpert).pScan = pScan;
        for i in 0..(*pIdxInfo).nConstraint {
            let mut pCons: *mut sqlite3_index_constraint =
                &mut *((*pIdxInfo).aConstraint).offset(i as isize) as *mut sqlite3_index_constraint;
            if (*pCons).usable as i32 != 0
                && (*pCons).iColumn >= 0
                && (*((*(*p).pTab).aCol).offset((*pCons).iColumn as isize)).iPk == 0
                && (*pCons).op as i32 & opmask != 0
            {
                let mut pNew: *mut IdxConstraint = 0 as *mut IdxConstraint;
                let mut zColl: *const i8 = sqlite3_vtab_collation(pIdxInfo, i);
                pNew = idxNewConstraint(&mut rc, zColl);
                if !pNew.is_null() {
                    (*pNew).iCol = (*pCons).iColumn;
                    if (*pCons).op as i32 == 2 {
                        (*pNew).pNext = (*pScan).pEq;
                        (*pScan).pEq = pNew;
                    } else {
                        (*pNew).bRange = 1;
                        (*pNew).pNext = (*pScan).pRange;
                        (*pScan).pRange = pNew;
                    }
                }
                n += 1;
                (*((*pIdxInfo).aConstraintUsage).offset(i as isize)).argvIndex = n;
            }
        }

        let mut i: i32 = (*pIdxInfo).nOrderBy - 1;
        while i >= 0 {
            let mut iCol: i32 = (*((*pIdxInfo).aOrderBy).offset(i as isize)).iColumn;
            if iCol >= 0 {
                let mut pNew_0: *mut IdxConstraint = idxNewConstraint(&mut rc, (*((*(*p).pTab).aCol).offset(iCol as isize)).zColl);
                if !pNew_0.is_null() {
                    (*pNew_0).iCol = iCol;
                    (*pNew_0).bDesc = (*((*pIdxInfo).aOrderBy).offset(i as isize)).desc as i32;
                    (*pNew_0).pNext = (*pScan).pOrder;
                    (*pNew_0).pLink = (*pScan).pOrder;
                    (*pScan).pOrder = pNew_0;
                    n += 1;
                }
            }
            i -= 1;
        }
    }
    (*pIdxInfo).estimatedCost = 1000000.0f64 / (n + 1) as f64;
    return rc;
}
unsafe extern "C" fn expertUpdate(
    mut _pVtab: *mut sqlite3_vtab,
    mut _nData: i32,
    mut _azData: *mut *mut sqlite3_value,
    mut _pRowid: *mut i64,
) -> i32 {
    return 0;
}
unsafe extern "C" fn expertOpen(mut pVTab: *mut sqlite3_vtab, mut ppCursor: *mut *mut sqlite3_vtab_cursor) -> i32 {
    let mut rc: i32 = 0;
    let mut pCsr: *mut ExpertCsr = 0 as *mut ExpertCsr;
    pCsr = idxMalloc(&mut rc, ::std::mem::size_of::<ExpertCsr>() as u64 as i32) as *mut ExpertCsr;
    *ppCursor = pCsr as *mut sqlite3_vtab_cursor;
    return rc;
}
unsafe extern "C" fn expertClose(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCsr: *mut ExpertCsr = cur as *mut ExpertCsr;
    sqlite3_finalize((*pCsr).pData);
    sqlite3_free(pCsr as *mut libc::c_void);
    return 0;
}
unsafe extern "C" fn expertEof(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCsr: *mut ExpertCsr = cur as *mut ExpertCsr;
    return ((*pCsr).pData == 0 as *mut sqlite3_stmt) as i32;
}
unsafe extern "C" fn expertNext(mut cur: *mut sqlite3_vtab_cursor) -> i32 {
    let mut pCsr: *mut ExpertCsr = cur as *mut ExpertCsr;
    let mut rc: i32 = 0;
    assert!(!((*pCsr).pData).is_null());
    rc = sqlite3_step((*pCsr).pData);
    if rc != 100 {
        rc = sqlite3_finalize((*pCsr).pData);
        (*pCsr).pData = 0 as *mut sqlite3_stmt;
    } else {
        rc = 0;
    }
    return rc;
}
unsafe extern "C" fn expertRowid(mut cur: *mut sqlite3_vtab_cursor, mut pRowid: *mut i64) -> i32 {
    *pRowid = 0;
    return 0;
}
unsafe extern "C" fn expertColumn(mut cur: *mut sqlite3_vtab_cursor, mut ctx: *mut sqlite3_context, mut i: i32) -> i32 {
    let mut pCsr: *mut ExpertCsr = cur as *mut ExpertCsr;
    let mut pVal: *mut sqlite3_value = 0 as *mut sqlite3_value;
    pVal = sqlite3_column_value((*pCsr).pData, i);
    if !pVal.is_null() {
        sqlite3_result_value(ctx, pVal);
    }
    return 0;
}
unsafe extern "C" fn expertFilter(
    mut cur: *mut sqlite3_vtab_cursor,
    mut idxNum: i32,
    mut idxStr: *const i8,
    mut argc: i32,
    mut argv: *mut *mut sqlite3_value,
) -> i32 {
    let mut pCsr: *mut ExpertCsr = cur as *mut ExpertCsr;
    let mut pVtab: *mut ExpertVtab = (*cur).pVtab as *mut ExpertVtab;
    let mut pExpert: *mut sqlite3expert = (*pVtab).pExpert;
    let mut rc: i32 = 0;
    rc = sqlite3_finalize((*pCsr).pData);
    (*pCsr).pData = 0 as *mut sqlite3_stmt;
    if rc == 0 {
        rc = idxPrintfPrepareStmt(
            (*pExpert).db,
            &mut (*pCsr).pData as *mut *mut sqlite3_stmt,
            &mut (*pVtab).base.zErrMsg as *mut *mut i8,
            b"SELECT * FROM main.%Q WHERE sample()\0" as *const u8 as *const i8,
            (*(*pVtab).pTab).zName,
        );
    }
    if rc == 0 {
        rc = expertNext(cur);
    }
    return rc;
}
unsafe extern "C" fn idxRegisterVtab(mut p: *mut sqlite3expert) -> i32 {
    const expertModule: sqlite3_module = sqlite3_module {
        iVersion: 2,
        xCreate: Some(expertConnect),
        xConnect: Some(expertConnect),
        xBestIndex: Some(expertBestIndex),
        xDisconnect: Some(expertDisconnect),
        xDestroy: Some(expertDisconnect),
        xOpen: Some(expertOpen),
        xClose: Some(expertClose),
        xFilter: Some(expertFilter),
        xNext: Some(expertNext),
        xEof: Some(expertEof),
        xColumn: Some(expertColumn),
        xRowid: Some(expertRowid),
        xUpdate: Some(expertUpdate),
        xBegin: None,
        xSync: None,
        xCommit: None,
        xRollback: None,
        xFindFunction: None,
        xRename: None,
        xSavepoint: None,
        xRelease: None,
        xRollbackTo: None,
        xShadowName: None,
    };
    return sqlite3_create_module(
        (*p).dbv,
        b"expert\0" as *const u8 as *const i8,
        &expertModule,
        p as *mut libc::c_void,
    );
}
unsafe extern "C" fn idxFinalize(mut pRc: *mut i32, mut pStmt: *mut sqlite3_stmt) {
    let mut rc: i32 = sqlite3_finalize(pStmt);
    if *pRc == 0 {
        *pRc = rc;
    }
}
unsafe extern "C" fn idxGetTableInfo(
    mut db: *mut sqlite3,
    mut zTab: *const i8,
    mut ppOut: *mut *mut IdxTable,
    mut pzErrmsg: *mut *mut i8,
) -> i32 {
    let mut p1: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut nCol: i32 = 0;
    let mut nTab: i32 = 0;
    let mut nByte: i32 = 0;
    let mut pNew: *mut IdxTable = 0 as *mut IdxTable;
    let mut rc: i32 = 0;
    let mut rc2: i32 = 0;
    let mut pCsr: *mut i8 = std::ptr::null_mut();
    let mut nPk: i32 = 0;
    *ppOut = 0 as *mut IdxTable;
    if zTab.is_null() {
        return 1;
    }
    nTab = strlen(zTab) as i32;
    nByte = (::std::mem::size_of::<IdxTable>() as u64)
        .wrapping_add(nTab as u64)
        .wrapping_add(1 as u64) as i32;
    rc = idxPrintfPrepareStmt(
        db,
        &mut p1 as *mut *mut sqlite3_stmt,
        pzErrmsg,
        b"PRAGMA table_xinfo=%Q\0" as *const u8 as *const i8,
        zTab,
    );
    while rc == 0 && 100 == sqlite3_step(p1) {
        let mut zCol: *const i8 = sqlite3_column_text(p1, 1) as *const i8;
        let mut zColSeq: *const i8 = 0 as *const i8;
        if zCol.is_null() {
            rc = 1;
            break;
        } else {
            nByte += 1 + strlen(zCol) as i32;
            rc = sqlite3_table_column_metadata(
                db,
                b"main\0" as *const u8 as *const i8,
                zTab,
                zCol,
                0 as *mut *const i8,
                &mut zColSeq,
                0 as *mut i32,
                0 as *mut i32,
                0 as *mut i32,
            );
            if zColSeq.is_null() {
                zColSeq = b"binary\0" as *const u8 as *const i8;
            }
            nByte += 1 + strlen(zColSeq) as i32;
            nCol += 1;
            nPk += (sqlite3_column_int(p1, 5) > 0) as i32;
        }
    }
    rc2 = sqlite3_reset(p1);
    if rc == 0 {
        rc = rc2;
    }
    nByte = (nByte as u64).wrapping_add((::std::mem::size_of::<IdxColumn>() as u64).wrapping_mul(nCol as u64)) as i32;
    if rc == 0 {
        pNew = idxMalloc(&mut rc, nByte) as *mut IdxTable;
    }
    if rc == 0 {
        (*pNew).aCol = &mut *pNew.offset(1) as *mut IdxTable as *mut IdxColumn;
        (*pNew).nCol = nCol;
        pCsr = &mut *((*pNew).aCol).offset(nCol as isize) as *mut IdxColumn as *mut i8;
    }
    nCol = 0;
    while rc == 0 && 100 == sqlite3_step(p1) {
        let mut zCol_0: *const i8 = sqlite3_column_text(p1, 1) as *const i8;
        let mut zColSeq_0: *const i8 = 0 as *const i8;
        let mut nCopy: i32 = 0;
        if zCol_0.is_null() {
            continue;
        }
        nCopy = strlen(zCol_0) as i32 + 1;
        (*((*pNew).aCol).offset(nCol as isize)).zName = pCsr;
        (*((*pNew).aCol).offset(nCol as isize)).iPk = (sqlite3_column_int(p1, 5) == 1 && nPk == 1) as i32;
        memcpy(pCsr as *mut libc::c_void, zCol_0 as *const libc::c_void, nCopy as u64);
        pCsr = pCsr.offset(nCopy as isize);
        rc = sqlite3_table_column_metadata(
            db,
            b"main\0" as *const u8 as *const i8,
            zTab,
            zCol_0,
            0 as *mut *const i8,
            &mut zColSeq_0,
            0 as *mut i32,
            0 as *mut i32,
            0 as *mut i32,
        );
        if rc == 0 {
            if zColSeq_0.is_null() {
                zColSeq_0 = b"binary\0" as *const u8 as *const i8;
            }
            nCopy = strlen(zColSeq_0) as i32 + 1;
            (*((*pNew).aCol).offset(nCol as isize)).zColl = pCsr;
            memcpy(pCsr as *mut libc::c_void, zColSeq_0 as *const libc::c_void, nCopy as u64);
            pCsr = pCsr.offset(nCopy as isize);
        }
        nCol += 1;
    }
    idxFinalize(&mut rc, p1);
    if rc != 0 {
        sqlite3_free(pNew as *mut libc::c_void);
        pNew = 0 as *mut IdxTable;
    } else if {
        assert!(!pNew.is_null());
        1
    } != 0
    {
        (*pNew).zName = pCsr;
        if {
            assert!(!((*pNew).zName).is_null());
            1
        } != 0
        {
            memcpy((*pNew).zName as *mut libc::c_void, zTab as *const libc::c_void, (nTab + 1) as u64);
        }
    }
    *ppOut = pNew;
    return rc;
}
unsafe extern "C" fn idxAppendText(mut pRc: *mut i32, mut zIn: *mut i8, mut zFmt: *const i8, mut args: ...) -> *mut i8 {
    let mut ap: ::std::ffi::VaListImpl;
    let mut zAppend: *mut i8 = std::ptr::null_mut();
    let mut zRet: *mut i8 = std::ptr::null_mut();
    let mut nIn: i32 = if !zIn.is_null() { strlen(zIn) as i32 } else { 0 };
    let mut nAppend: i32 = 0;
    ap = args.clone();
    if *pRc == 0 {
        zAppend = sqlite3_vmprintf(zFmt, ap.as_va_list());
        if !zAppend.is_null() {
            nAppend = strlen(zAppend) as i32;
            zRet = sqlite3_malloc(nIn + nAppend + 1) as *mut i8;
        }
        if !zAppend.is_null() && !zRet.is_null() {
            if nIn != 0 {
                memcpy(zRet as *mut libc::c_void, zIn as *const libc::c_void, nIn as u64);
            }
            memcpy(
                &mut *zRet.offset(nIn as isize) as *mut i8 as *mut libc::c_void,
                zAppend as *const libc::c_void,
                (nAppend + 1) as u64,
            );
        } else {
            sqlite3_free(zRet as *mut libc::c_void);
            zRet = std::ptr::null_mut();
            *pRc = 7 as i32;
        }
        sqlite3_free(zAppend as *mut libc::c_void);
        sqlite3_free(zIn as *mut libc::c_void);
    }
    return zRet;
}
unsafe extern "C" fn idxIdentifierRequiresQuotes(mut zId: *const i8) -> i32 {
    let mut i: i32 = 0;
    i = 0;
    while *zId.offset(i as isize) != 0 {
        if !(*zId.offset(i as isize) as i32 == '_' as i32)
            && !(*zId.offset(i as isize) as i32 >= '0' as i32 && *zId.offset(i as isize) as i32 <= '9' as i32)
            && !(*zId.offset(i as isize) as i32 >= 'a' as i32 && *zId.offset(i as isize) as i32 <= 'z' as i32)
            && !(*zId.offset(i as isize) as i32 >= 'A' as i32 && *zId.offset(i as isize) as i32 <= 'Z' as i32)
        {
            return 1;
        }
        i += 1;
    }
    return 0;
}
unsafe extern "C" fn idxAppendColDefn(
    mut pRc: *mut i32,
    mut zIn: *mut i8,
    mut pTab: *mut IdxTable,
    mut pCons: *mut IdxConstraint,
) -> *mut i8 {
    let mut zRet: *mut i8 = zIn;
    let mut p: *mut IdxColumn = &mut *((*pTab).aCol).offset((*pCons).iCol as isize) as *mut IdxColumn;
    if !zRet.is_null() {
        zRet = idxAppendText(pRc, zRet, b", \0" as *const u8 as *const i8);
    }
    if idxIdentifierRequiresQuotes((*p).zName) != 0 {
        zRet = idxAppendText(pRc, zRet, b"%Q\0" as *const u8 as *const i8, (*p).zName);
    } else {
        zRet = idxAppendText(pRc, zRet, b"%s\0" as *const u8 as *const i8, (*p).zName);
    }
    if sqlite3_stricmp((*p).zColl, (*pCons).zColl) != 0 {
        if idxIdentifierRequiresQuotes((*pCons).zColl) != 0 {
            zRet = idxAppendText(pRc, zRet, b" COLLATE %Q\0" as *const u8 as *const i8, (*pCons).zColl);
        } else {
            zRet = idxAppendText(pRc, zRet, b" COLLATE %s\0" as *const u8 as *const i8, (*pCons).zColl);
        }
    }
    if (*pCons).bDesc != 0 {
        zRet = idxAppendText(pRc, zRet, b" DESC\0" as *const u8 as *const i8);
    }
    return zRet;
}
unsafe extern "C" fn idxFindCompatible(
    mut pRc: *mut i32,
    mut dbm: *mut sqlite3,
    mut pScan: *mut IdxScan,
    mut pEq: *mut IdxConstraint,
    mut pTail: *mut IdxConstraint,
) -> i32 {
    let mut zTbl: *const i8 = (*(*pScan).pTab).zName;
    let mut pIdxList: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut pIter: *mut IdxConstraint = 0 as *mut IdxConstraint;
    let mut nEq: i32 = 0;
    let mut rc: i32 = 0;
    pIter = pEq;
    while !pIter.is_null() {
        nEq += 1;
        pIter = (*pIter).pLink;
    }
    rc = idxPrintfPrepareStmt(
        dbm,
        &mut pIdxList as *mut *mut sqlite3_stmt,
        0 as *mut *mut i8,
        b"PRAGMA index_list=%Q\0" as *const u8 as *const i8,
        zTbl,
    );
    while rc == 0 && sqlite3_step(pIdxList) == 100 {
        let mut bMatch: i32 = 1;
        let mut pT: *mut IdxConstraint = pTail;
        let mut pInfo: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
        let mut zIdx: *const i8 = sqlite3_column_text(pIdxList, 1) as *const i8;
        if zIdx.is_null() {
            continue;
        }
        pIter = pEq;
        while !pIter.is_null() {
            (*pIter).bFlag = 0;
            pIter = (*pIter).pLink;
        }
        rc = idxPrintfPrepareStmt(
            dbm,
            &mut pInfo as *mut *mut sqlite3_stmt,
            0 as *mut *mut i8,
            b"PRAGMA index_xInfo=%Q\0" as *const u8 as *const i8,
            zIdx,
        );
        while rc == 0 && sqlite3_step(pInfo) == 100 {
            let mut iIdx: i32 = sqlite3_column_int(pInfo, 0);
            let mut iCol: i32 = sqlite3_column_int(pInfo, 1);
            let mut zColl: *const i8 = sqlite3_column_text(pInfo, 4) as *const i8;
            if iIdx < nEq {
                pIter = pEq;
                while !pIter.is_null() {
                    if !((*pIter).bFlag != 0) {
                        if !((*pIter).iCol != iCol) {
                            if !(sqlite3_stricmp((*pIter).zColl, zColl) != 0) {
                                (*pIter).bFlag = 1;
                                break;
                            }
                        }
                    }
                    pIter = (*pIter).pLink;
                }
                if !pIter.is_null() {
                    continue;
                }
                bMatch = 0;
                break;
            } else {
                if pT.is_null() {
                    continue;
                }
                if (*pT).iCol != iCol || sqlite3_stricmp((*pT).zColl, zColl) != 0 {
                    bMatch = 0;
                    break;
                } else {
                    pT = (*pT).pLink;
                }
            }
        }
        idxFinalize(&mut rc, pInfo);
        if rc == 0 && bMatch != 0 {
            sqlite3_finalize(pIdxList);
            return 1;
        }
    }
    idxFinalize(&mut rc, pIdxList);
    *pRc = rc;
    return 0;
}
unsafe extern "C" fn countNonzeros(
    mut pCount: *mut libc::c_void,
    mut nc: i32,
    mut azResults: *mut *mut i8,
    mut azColumns: *mut *mut i8,
) -> i32 {
    if nc > 0 && (*(*azResults.offset(0)).offset(0) as i32 != '0' as i32 || *(*azResults.offset(0)).offset(1) as i32 != 0) {
        *(pCount as *mut i32) += 1;
    }
    return 0;
}
unsafe extern "C" fn idxCreateFromCons(
    mut p: *mut sqlite3expert,
    mut pScan: *mut IdxScan,
    mut pEq: *mut IdxConstraint,
    mut pTail: *mut IdxConstraint,
) -> i32 {
    let mut dbm: *mut sqlite3 = (*p).dbm;
    let mut rc: i32 = 0;
    if (!pEq.is_null() || !pTail.is_null()) && 0 == idxFindCompatible(&mut rc, dbm, pScan, pEq, pTail) {
        let mut pTab: *mut IdxTable = (*pScan).pTab;
        let mut zCols: *mut i8 = std::ptr::null_mut();
        let mut zIdx: *mut i8 = std::ptr::null_mut();
        let mut pCons: *mut IdxConstraint = 0 as *mut IdxConstraint;
        let mut h: u32 = 0;
        let mut zFmt: *const i8 = 0 as *const i8;
        pCons = pEq;
        while !pCons.is_null() {
            zCols = idxAppendColDefn(&mut rc, zCols, pTab, pCons);
            pCons = (*pCons).pLink;
        }
        pCons = pTail;
        while !pCons.is_null() {
            zCols = idxAppendColDefn(&mut rc, zCols, pTab, pCons);
            pCons = (*pCons).pLink;
        }
        if rc == 0 {
            let mut zTable: *const i8 = (*(*pScan).pTab).zName;
            let mut quoteTable: i32 = idxIdentifierRequiresQuotes(zTable);
            let mut zName: *mut i8 = std::ptr::null_mut();
            let mut collisions: i32 = 0;
            loop {
                let mut i: i32 = 0;
                let mut zFind: *mut i8 = std::ptr::null_mut();
                i = 0;
                while *zCols.offset(i as isize) != 0 {
                    h = h.wrapping_add((h << 3).wrapping_add(*zCols.offset(i as isize) as u32));
                    i += 1;
                }
                sqlite3_free(zName as *mut libc::c_void);
                zName = sqlite3_mprintf(b"%s_idx_%08x\0" as *const u8 as *const i8, zTable, h);
                if zName.is_null() {
                    break;
                }
                zFmt = b"SELECT count(*) FROM sqlite_schema WHERE name=%Q AND type in ('index','table','view')\0" as *const u8 as *const i8;
                zFind = sqlite3_mprintf(zFmt, zName);
                i = 0;
                rc = sqlite3_exec(
                    dbm,
                    zFind,
                    Some(countNonzeros),
                    &mut i as *mut i32 as *mut libc::c_void,
                    0 as *mut *mut i8,
                );
                assert!(rc == 0);
                sqlite3_free(zFind as *mut libc::c_void);
                if i == 0 {
                    collisions = 0;
                    break;
                } else {
                    collisions += 1;
                    if !(collisions < 50 && !zName.is_null()) {
                        break;
                    }
                }
            }
            if collisions != 0 {
                rc = 5 | (3) << 8;
            } else if zName.is_null() {
                rc = 7 as i32;
            } else {
                if quoteTable != 0 {
                    zFmt = b"CREATE INDEX \"%w\" ON \"%w\"(%s)\0" as *const u8 as *const i8;
                } else {
                    zFmt = b"CREATE INDEX %s ON %s(%s)\0" as *const u8 as *const i8;
                }
                zIdx = sqlite3_mprintf(zFmt, zName, zTable, zCols);
                if zIdx.is_null() {
                    rc = 7 as i32;
                } else {
                    rc = sqlite3_exec(dbm, zIdx, None, std::ptr::null_mut(), (*p).pzErrmsg);
                    if rc != 0 {
                        rc = 5 | (3) << 8;
                    } else {
                        idxHashAdd(&mut rc, &mut (*p).hIdx, zName, zIdx);
                    }
                }
                sqlite3_free(zName as *mut libc::c_void);
                sqlite3_free(zIdx as *mut libc::c_void);
            }
        }
        sqlite3_free(zCols as *mut libc::c_void);
    }
    return rc;
}
unsafe extern "C" fn idxFindConstraint(mut pList: *mut IdxConstraint, mut p: *mut IdxConstraint) -> i32 {
    let mut pCmp: *mut IdxConstraint = 0 as *mut IdxConstraint;
    pCmp = pList;
    while !pCmp.is_null() {
        if (*p).iCol == (*pCmp).iCol {
            return 1;
        }
        pCmp = (*pCmp).pLink;
    }
    return 0;
}
unsafe extern "C" fn idxCreateFromWhere(mut p: *mut sqlite3expert, mut pScan: *mut IdxScan, mut pTail: *mut IdxConstraint) -> i32 {
    let mut p1: *mut IdxConstraint = 0 as *mut IdxConstraint;
    let mut pCon: *mut IdxConstraint = 0 as *mut IdxConstraint;
    let mut rc: i32 = 0;
    pCon = (*pScan).pEq;
    while !pCon.is_null() {
        if idxFindConstraint(p1, pCon) == 0 && idxFindConstraint(pTail, pCon) == 0 {
            (*pCon).pLink = p1;
            p1 = pCon;
        }
        pCon = (*pCon).pNext;
    }
    rc = idxCreateFromCons(p, pScan, p1, pTail);
    if pTail.is_null() {
        pCon = (*pScan).pRange;
        while rc == 0 && !pCon.is_null() {
            assert!(((*pCon).pLink).is_null());
            if idxFindConstraint(p1, pCon) == 0 && idxFindConstraint(pTail, pCon) == 0 {
                rc = idxCreateFromCons(p, pScan, p1, pCon);
            }
            pCon = (*pCon).pNext;
        }
    }
    return rc;
}
unsafe extern "C" fn idxCreateCandidates(mut p: *mut sqlite3expert) -> i32 {
    let mut rc: i32 = 0;
    let mut pIter: *mut IdxScan = 0 as *mut IdxScan;
    pIter = (*p).pScan;
    while !pIter.is_null() && rc == 0 {
        rc = idxCreateFromWhere(p, pIter, 0 as *mut IdxConstraint);
        if rc == 0 && !((*pIter).pOrder).is_null() {
            rc = idxCreateFromWhere(p, pIter, (*pIter).pOrder);
        }
        pIter = (*pIter).pNextScan;
    }
    return rc;
}
unsafe extern "C" fn idxConstraintFree(mut pConstraint: *mut IdxConstraint) {
    let mut pNext: *mut IdxConstraint = 0 as *mut IdxConstraint;
    let mut p: *mut IdxConstraint = 0 as *mut IdxConstraint;
    p = pConstraint;
    while !p.is_null() {
        pNext = (*p).pNext;
        sqlite3_free(p as *mut libc::c_void);
        p = pNext;
    }
}
unsafe extern "C" fn idxScanFree(mut pScan: *mut IdxScan, mut pLast: *mut IdxScan) {
    let mut p: *mut IdxScan = 0 as *mut IdxScan;
    let mut pNext: *mut IdxScan = 0 as *mut IdxScan;
    p = pScan;
    while p != pLast {
        pNext = (*p).pNextScan;
        idxConstraintFree((*p).pOrder);
        idxConstraintFree((*p).pEq);
        idxConstraintFree((*p).pRange);
        sqlite3_free(p as *mut libc::c_void);
        p = pNext;
    }
}
unsafe fn idxStatementFree(mut pStatement: *mut IdxStatement, mut pLast: *mut IdxStatement) {
    let mut p: *mut IdxStatement = 0 as *mut IdxStatement;
    let mut pNext: *mut IdxStatement = 0 as *mut IdxStatement;
    p = pStatement;
    while p != pLast {
        pNext = (*p).pNext;
        sqlite3_free((*p).zEQP as *mut libc::c_void);
        sqlite3_free((*p).zIdx as *mut libc::c_void);
        sqlite3_free(p as *mut libc::c_void);
        p = pNext;
    }
}
unsafe fn idxTableFree(mut pTab: *mut IdxTable) {
    let mut pIter: *mut IdxTable = 0 as *mut IdxTable;
    let mut pNext: *mut IdxTable = 0 as *mut IdxTable;
    pIter = pTab;
    while !pIter.is_null() {
        pNext = (*pIter).pNext;
        sqlite3_free(pIter as *mut libc::c_void);
        pIter = pNext;
    }
}
unsafe fn idxWriteFree(mut pTab: *mut IdxWrite) {
    let mut pIter: *mut IdxWrite = 0 as *mut IdxWrite;
    let mut pNext: *mut IdxWrite = 0 as *mut IdxWrite;
    pIter = pTab;
    while !pIter.is_null() {
        pNext = (*pIter).pNext;
        sqlite3_free(pIter as *mut libc::c_void);
        pIter = pNext;
    }
}
unsafe extern "C" fn idxFindIndexes(mut p: *mut sqlite3expert, mut pzErr: *mut *mut i8) -> i32 {
    let mut pStmt: *mut IdxStatement = 0 as *mut IdxStatement;
    let mut dbm: *mut sqlite3 = (*p).dbm;
    let mut rc: i32 = 0;
    let mut hIdx: IdxHash = IdxHash {
        pFirst: 0 as *mut IdxHashEntry,
        aHash: [0 as *mut IdxHashEntry; 1023],
    };
    idxHashInit(&mut hIdx);
    pStmt = (*p).pStatement;
    's_19: while rc == 0 && !pStmt.is_null() {
        let mut pEntry: *mut IdxHashEntry = 0 as *mut IdxHashEntry;
        let mut pExplain: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
        idxHashClear(&mut hIdx);
        rc = idxPrintfPrepareStmt(
            dbm,
            &mut pExplain as *mut *mut sqlite3_stmt,
            pzErr,
            b"EXPLAIN QUERY PLAN %s\0" as *const u8 as *const i8,
            (*pStmt).zSql,
        );
        while rc == 0 && sqlite3_step(pExplain) == 100 {
            let mut zDetail: *const i8 = sqlite3_column_text(pExplain, 3) as *const i8;
            let mut nDetail: i32 = 0;
            if zDetail.is_null() {
                continue;
            }
            nDetail = strlen(zDetail) as i32;
            for i in 0..nDetail {
                let mut zIdx: *const i8 = 0 as *const i8;
                if (i + 13) < nDetail
                    && memcmp(
                        &*zDetail.offset(i as isize) as *const i8 as *const libc::c_void,
                        b" USING INDEX \0" as *const u8 as *const i8 as *const libc::c_void,
                        13 as u64,
                    ) == 0
                {
                    zIdx = &*zDetail.offset((i + 13) as isize) as *const i8;
                } else if (i + 22) < nDetail
                    && memcmp(
                        &*zDetail.offset(i as isize) as *const i8 as *const libc::c_void,
                        b" USING COVERING INDEX \0" as *const u8 as *const i8 as *const libc::c_void,
                        22 as u64,
                    ) == 0
                {
                    zIdx = &*zDetail.offset((i + 22) as isize) as *const i8;
                }
                if !zIdx.is_null() {
                    let mut zSql: *const i8 = 0 as *const i8;
                    let mut nIdx: i32 = 0;
                    while *zIdx.offset(nIdx as isize) as i32 != '\0' as i32
                        && (*zIdx.offset(nIdx as isize) as i32 != ' ' as i32 || *zIdx.offset((nIdx + 1) as isize) as i32 != '(' as i32)
                    {
                        nIdx += 1;
                    }
                    zSql = idxHashSearch(&mut (*p).hIdx, zIdx, nIdx);
                    if zSql.is_null() {
                        break;
                    }
                    idxHashAdd(&mut rc, &mut hIdx, zSql, 0 as *const i8);
                    if rc != 0 {
                        break 's_19;
                    } else {
                        break;
                    }
                }
            }
            if *zDetail.offset(0) as i32 != '-' as i32 {
                (*pStmt).zEQP = idxAppendText(&mut rc as *mut i32, (*pStmt).zEQP, b"%s\n\0" as *const u8 as *const i8, zDetail);
            }
        }
        pEntry = hIdx.pFirst;
        while !pEntry.is_null() {
            (*pStmt).zIdx = idxAppendText(
                &mut rc as *mut i32,
                (*pStmt).zIdx,
                b"%s;\n\0" as *const u8 as *const i8,
                (*pEntry).zKey,
            );
            pEntry = (*pEntry).pNext;
        }
        idxFinalize(&mut rc, pExplain);
        pStmt = (*pStmt).pNext;
    }
    idxHashClear(&mut hIdx);
    return rc;
}
unsafe extern "C" fn idxAuthCallback(
    mut pCtx: *mut libc::c_void,
    mut eOp: i32,
    mut z3: *const i8,
    mut z4: *const i8,
    mut zDb: *const i8,
    mut zTrigger: *const i8,
) -> i32 {
    let mut rc: i32 = 0;
    if eOp == 18 || eOp == 23 || eOp == 9 {
        if sqlite3_stricmp(zDb, b"main\0" as *const u8 as *const i8) == 0 {
            let mut p: *mut sqlite3expert = pCtx as *mut sqlite3expert;
            let mut pTab: *mut IdxTable = 0 as *mut IdxTable;
            pTab = (*p).pTable;
            while !pTab.is_null() {
                if 0 == sqlite3_stricmp(z3, (*pTab).zName) {
                    break;
                }
                pTab = (*pTab).pNext;
            }
            if !pTab.is_null() {
                let mut pWrite: *mut IdxWrite = 0 as *mut IdxWrite;
                pWrite = (*p).pWrite;
                while !pWrite.is_null() {
                    if (*pWrite).pTab == pTab && (*pWrite).eOp == eOp {
                        break;
                    }
                    pWrite = (*pWrite).pNext;
                }
                if pWrite.is_null() {
                    pWrite = idxMalloc(&mut rc, ::std::mem::size_of::<IdxWrite>() as u64 as i32) as *mut IdxWrite;
                    if rc == 0 {
                        (*pWrite).pTab = pTab;
                        (*pWrite).eOp = eOp;
                        (*pWrite).pNext = (*p).pWrite;
                        (*p).pWrite = pWrite;
                    }
                }
            }
        }
    }
    return rc;
}
unsafe extern "C" fn idxProcessOneTrigger(mut p: *mut sqlite3expert, mut pWrite: *mut IdxWrite, mut pzErr: *mut *mut i8) -> i32 {
    const zInt: *const i8 = b"t592690916721053953805701627921227776\0" as *const u8 as *const i8;
    const zDrop: *const i8 = b"DROP TABLE t592690916721053953805701627921227776\0" as *const u8 as *const i8;
    let mut pTab: *mut IdxTable = (*pWrite).pTab;
    let mut zTab: *const i8 = (*pTab).zName;
    let mut zSql: *const i8 =
        b"SELECT 'CREATE TEMP' || substr(sql, 7) FROM sqlite_schema WHERE tbl_name = %Q AND type IN ('table', 'trigger') ORDER BY type;\0"
            as *const u8 as *const i8;
    let mut pSelect: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut rc: i32 = 0;
    let mut zWrite: *mut i8 = std::ptr::null_mut();
    rc = idxPrintfPrepareStmt((*p).db, &mut pSelect as *mut *mut sqlite3_stmt, pzErr, zSql, zTab, zTab);
    while rc == 0 && 100 == sqlite3_step(pSelect) {
        let mut zCreate: *const i8 = sqlite3_column_text(pSelect, 0) as *const i8;
        if zCreate.is_null() {
            continue;
        }
        rc = sqlite3_exec((*p).dbv, zCreate, None, std::ptr::null_mut(), pzErr);
    }
    idxFinalize(&mut rc, pSelect);
    if rc == 0 {
        let mut z: *mut i8 = sqlite3_mprintf(b"ALTER TABLE temp.%Q RENAME TO %Q\0" as *const u8 as *const i8, zTab, zInt);
        if z.is_null() {
            rc = 7 as i32;
        } else {
            rc = sqlite3_exec((*p).dbv, z, None, std::ptr::null_mut(), pzErr);
            sqlite3_free(z as *mut libc::c_void);
        }
    }
    match (*pWrite).eOp {
        18 => {
            zWrite = idxAppendText(
                &mut rc as *mut i32,
                zWrite,
                b"INSERT INTO %Q VALUES(\0" as *const u8 as *const i8,
                zInt,
            );
            for i in 0..(*pTab).nCol {
                zWrite = idxAppendText(
                    &mut rc as *mut i32,
                    zWrite,
                    b"%s?\0" as *const u8 as *const i8,
                    if i == 0 {
                        b"\0" as *const u8 as *const i8
                    } else {
                        b", \0" as *const u8 as *const i8
                    },
                );
            }
            zWrite = idxAppendText(&mut rc as *mut i32, zWrite, b")\0" as *const u8 as *const i8);
        }
        23 => {
            zWrite = idxAppendText(&mut rc as *mut i32, zWrite, b"UPDATE %Q SET \0" as *const u8 as *const i8, zInt);
            for i_0 in 0..(*pTab).nCol {
                zWrite = idxAppendText(
                    &mut rc as *mut i32,
                    zWrite,
                    b"%s%Q=?\0" as *const u8 as *const i8,
                    if i_0 == 0 {
                        b"\0" as *const u8 as *const i8
                    } else {
                        b", \0" as *const u8 as *const i8
                    },
                    (*((*pTab).aCol).offset(i_0 as isize)).zName,
                );
            }
        }
        _ => {
            assert!((*pWrite).eOp == 9);
            if rc == 0 {
                zWrite = sqlite3_mprintf(b"DELETE FROM %Q\0" as *const u8 as *const i8, zInt);
                if zWrite.is_null() {
                    rc = 7 as i32;
                }
            }
        }
    }
    if rc == 0 {
        let mut pX: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
        rc = sqlite3_prepare_v2((*p).dbv, zWrite, -1, &mut pX, 0 as *mut *const i8);
        idxFinalize(&mut rc, pX);
        if rc != 0 {
            idxDatabaseError((*p).dbv, pzErr);
        }
    }
    sqlite3_free(zWrite as *mut libc::c_void);
    if rc == 0 {
        rc = sqlite3_exec((*p).dbv, zDrop, None, std::ptr::null_mut(), pzErr);
    }
    return rc;
}
unsafe extern "C" fn idxProcessTriggers(mut p: *mut sqlite3expert, mut pzErr: *mut *mut i8) -> i32 {
    let mut rc: i32 = 0;
    let mut pEnd: *mut IdxWrite = 0 as *mut IdxWrite;
    let mut pFirst: *mut IdxWrite = (*p).pWrite;
    while rc == 0 && pFirst != pEnd {
        let mut pIter: *mut IdxWrite = 0 as *mut IdxWrite;
        pIter = pFirst;
        while rc == 0 && pIter != pEnd {
            rc = idxProcessOneTrigger(p, pIter, pzErr);
            pIter = (*pIter).pNext;
        }
        pEnd = pFirst;
        pFirst = (*p).pWrite;
    }
    return rc;
}
unsafe extern "C" fn idxCreateVtabSchema(mut p: *mut sqlite3expert, mut pzErrmsg: *mut *mut i8) -> i32 {
    let mut rc: i32 = idxRegisterVtab(p);
    let mut pSchema: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    rc = idxPrepareStmt(
        (*p).db,
        &mut pSchema,
        pzErrmsg,
        b"SELECT type, name, sql, 1 FROM sqlite_schema WHERE type IN ('table','view') AND name NOT LIKE 'sqlite_%%'  UNION ALL SELECT type, name, sql, 2 FROM sqlite_schema WHERE type = 'trigger'  AND tbl_name IN(SELECT name FROM sqlite_schema WHERE type = 'view') ORDER BY 4, 1\0"
            as *const u8 as *const i8,
    );
    while rc == 0 && 100 == sqlite3_step(pSchema) {
        let mut zType: *const i8 = sqlite3_column_text(pSchema, 0) as *const i8;
        let mut zName: *const i8 = sqlite3_column_text(pSchema, 1) as *const i8;
        let mut zSql: *const i8 = sqlite3_column_text(pSchema, 2) as *const i8;
        if zType.is_null() || zName.is_null() {
            continue;
        }
        if *zType.offset(0) as i32 == 'v' as i32 || *zType.offset(1) as i32 == 'r' as i32 {
            if !zSql.is_null() {
                rc = sqlite3_exec((*p).dbv, zSql, None, std::ptr::null_mut(), pzErrmsg);
            }
        } else {
            let mut pTab: *mut IdxTable = 0 as *mut IdxTable;
            rc = idxGetTableInfo((*p).db, zName, &mut pTab, pzErrmsg);
            if rc == 0 {
                let mut zInner: *mut i8 = std::ptr::null_mut();
                let mut zOuter: *mut i8 = std::ptr::null_mut();
                (*pTab).pNext = (*p).pTable;
                (*p).pTable = pTab;
                zInner = idxAppendText(
                    &mut rc as *mut i32,
                    std::ptr::null_mut(),
                    b"CREATE TABLE x(\0" as *const u8 as *const i8,
                );
                for i in 0..(*pTab).nCol {
                    zInner = idxAppendText(
                        &mut rc as *mut i32,
                        zInner,
                        b"%s%Q COLLATE %s\0" as *const u8 as *const i8,
                        if i == 0 {
                            b"\0" as *const u8 as *const i8
                        } else {
                            b", \0" as *const u8 as *const i8
                        },
                        (*((*pTab).aCol).offset(i as isize)).zName,
                        (*((*pTab).aCol).offset(i as isize)).zColl,
                    );
                }
                zInner = idxAppendText(&mut rc as *mut i32, zInner, b")\0" as *const u8 as *const i8);
                zOuter = idxAppendText(
                    &mut rc as *mut i32,
                    std::ptr::null_mut(),
                    b"CREATE VIRTUAL TABLE %Q USING expert(%Q)\0" as *const u8 as *const i8,
                    zName,
                    zInner,
                );
                if rc == 0 {
                    rc = sqlite3_exec((*p).dbv, zOuter, None, std::ptr::null_mut(), pzErrmsg);
                }
                sqlite3_free(zInner as *mut libc::c_void);
                sqlite3_free(zOuter as *mut libc::c_void);
            }
        }
    }
    idxFinalize(&mut rc, pSchema);
    return rc;
}
unsafe extern "C" fn idxSampleFunc(mut pCtx: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut p: *mut IdxSampleCtx = sqlite3_user_data(pCtx) as *mut IdxSampleCtx;
    let mut bRet: i32 = 0;
    assert!(argc == 0);
    if (*p).nRow == 0.0f64 {
        bRet = 1;
    } else {
        bRet = ((*p).nRet / (*p).nRow <= (*p).target) as i32;
        if bRet == 0 {
            let mut rnd: u16 = 0;
            sqlite3_randomness(2, &mut rnd as *mut u16 as *mut libc::c_void);
            bRet = (rnd as i32 % 100 <= (*p).iTarget) as i32;
        }
    }
    sqlite3_result_int(pCtx, bRet);
    (*p).nRow += 1.0f64;
    (*p).nRet += bRet as f64;
}
unsafe extern "C" fn idxRemFunc(mut pCtx: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut p: *mut IdxRemCtx = sqlite3_user_data(pCtx) as *mut IdxRemCtx;
    let mut pSlot: *mut IdxRemSlot = 0 as *mut IdxRemSlot;
    let mut iSlot: i32 = 0;
    assert!(argc == 2);
    iSlot = sqlite3_value_int(*argv.offset(0));
    assert!(iSlot <= (*p).nSlot);
    pSlot = &mut *((*p).aSlot).as_mut_ptr().offset(iSlot as isize) as *mut IdxRemSlot;
    match (*pSlot).eType {
        1 => {
            sqlite3_result_int64(pCtx, (*pSlot).iVal);
        }
        2 => {
            sqlite3_result_double(pCtx, (*pSlot).rVal);
        }
        4 => {
            sqlite3_result_blob(
                pCtx,
                (*pSlot).z as *const libc::c_void,
                (*pSlot).n,
                ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
            );
        }
        3 => {
            sqlite3_result_text(
                pCtx,
                (*pSlot).z,
                (*pSlot).n,
                ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
            );
        }
        5 | _ => {}
    }
    (*pSlot).eType = sqlite3_value_type(*argv.offset(1));
    match (*pSlot).eType {
        1 => {
            (*pSlot).iVal = sqlite3_value_int64(*argv.offset(1));
        }
        2 => {
            (*pSlot).rVal = sqlite3_value_double(*argv.offset(1));
        }
        4 | 3 => {
            let mut nByte: i32 = sqlite3_value_bytes(*argv.offset(1));
            let mut pData: *const libc::c_void = std::ptr::null();
            if nByte > (*pSlot).nByte {
                let mut zNew: *mut i8 = sqlite3_realloc((*pSlot).z as *mut libc::c_void, nByte * 2) as *mut i8;
                if zNew.is_null() {
                    sqlite3_result_error_nomem(pCtx);
                    return;
                }
                (*pSlot).nByte = nByte * 2;
                (*pSlot).z = zNew;
            }
            (*pSlot).n = nByte;
            if (*pSlot).eType == 4 {
                pData = sqlite3_value_blob(*argv.offset(1));
                if !pData.is_null() {
                    memcpy((*pSlot).z as *mut libc::c_void, pData, nByte as u64);
                }
            } else {
                pData = sqlite3_value_text(*argv.offset(1)) as *const libc::c_void;
                memcpy((*pSlot).z as *mut libc::c_void, pData, nByte as u64);
            }
        }
        5 | _ => {}
    };
}
unsafe extern "C" fn idxLargestIndex(mut db: *mut sqlite3, mut pnMax: *mut i32, mut pzErr: *mut *mut i8) -> i32 {
    let mut rc: i32 = 0;
    let mut zMax: *const i8 = b"SELECT max(i.seqno) FROM   sqlite_schema AS s,   pragma_index_list(s.name) AS l,   pragma_index_info(l.name) AS i WHERE s.type = 'table'\0"
        as *const u8 as *const i8;
    let mut pMax: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    *pnMax = 0;
    rc = idxPrepareStmt(db, &mut pMax, pzErr, zMax);
    if rc == 0 && 100 == sqlite3_step(pMax) {
        *pnMax = sqlite3_column_int(pMax, 0) + 1;
    }
    idxFinalize(&mut rc, pMax);
    return rc;
}
unsafe extern "C" fn idxPopulateOneStat1(
    mut p: *mut sqlite3expert,
    mut pIndexXInfo: *mut sqlite3_stmt,
    mut pWriteStat: *mut sqlite3_stmt,
    mut zTab: *const i8,
    mut zIdx: *const i8,
    mut pzErr: *mut *mut i8,
) -> i32 {
    let mut zCols: *mut i8 = std::ptr::null_mut();
    let mut zOrder: *mut i8 = std::ptr::null_mut();
    let mut zQuery: *mut i8 = std::ptr::null_mut();
    let mut nCol: i32 = 0;
    let mut i: i32 = 0;
    let mut pQuery: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut aStat: *mut i32 = 0 as *mut i32;
    let mut rc: i32 = 0;
    assert!((*p).iSample > 0);
    sqlite3_bind_text(pIndexXInfo, 1, zIdx, -1, None);
    while 0 == rc && 100 == sqlite3_step(pIndexXInfo) {
        let mut zComma: *const i8 = if zCols.is_null() {
            b"\0" as *const u8 as *const i8
        } else {
            b", \0" as *const u8 as *const i8
        };
        let mut zName: *const i8 = sqlite3_column_text(pIndexXInfo, 0) as *const i8;
        let mut zColl: *const i8 = sqlite3_column_text(pIndexXInfo, 1) as *const i8;
        zCols = idxAppendText(
            &mut rc as *mut i32,
            zCols,
            b"%sx.%Q IS rem(%d, x.%Q) COLLATE %s\0" as *const u8 as *const i8,
            zComma,
            zName,
            nCol,
            zName,
            zColl,
        );
        nCol += 1;
        zOrder = idxAppendText(&mut rc as *mut i32, zOrder, b"%s%d\0" as *const u8 as *const i8, zComma, nCol);
    }
    sqlite3_reset(pIndexXInfo);
    if rc == 0 {
        if (*p).iSample == 100 {
            zQuery = sqlite3_mprintf(b"SELECT %s FROM %Q x ORDER BY %s\0" as *const u8 as *const i8, zCols, zTab, zOrder);
        } else {
            zQuery = sqlite3_mprintf(
                b"SELECT %s FROM temp.t592690916721053953805701627921227776 x ORDER BY %s\0" as *const u8 as *const i8,
                zCols,
                zOrder,
            );
        }
    }
    sqlite3_free(zCols as *mut libc::c_void);
    sqlite3_free(zOrder as *mut libc::c_void);
    if rc == 0 {
        let mut dbrem: *mut sqlite3 = if (*p).iSample == 100 { (*p).db } else { (*p).dbv };
        rc = idxPrepareStmt(dbrem, &mut pQuery, pzErr, zQuery);
    }
    sqlite3_free(zQuery as *mut libc::c_void);
    if rc == 0 {
        aStat = idxMalloc(
            &mut rc,
            (::std::mem::size_of::<i32>() as u64).wrapping_mul((nCol + 1) as u64) as i32,
        ) as *mut i32;
    }
    if rc == 0 && 100 == sqlite3_step(pQuery) {
        let mut pEntry: *mut IdxHashEntry = 0 as *mut IdxHashEntry;
        let mut zStat: *mut i8 = std::ptr::null_mut();
        for i in 0..=nCol {
            *aStat.offset(i as isize) = 1;
        }
        while rc == 0 && 100 == sqlite3_step(pQuery) {
            *aStat.offset(0) += 1;
            i = 0;
            while i < nCol {
                if sqlite3_column_int(pQuery, i) == 0 {
                    break;
                }
                i += 1;
            }
            while i < nCol {
                *aStat.offset((i + 1) as isize) += 1;
                i += 1;
            }
        }
        if rc == 0 {
            let mut s0: i32 = *aStat.offset(0);
            zStat = sqlite3_mprintf(b"%d\0" as *const u8 as *const i8, s0);
            if zStat.is_null() {
                rc = 7 as i32;
            }
            for i in 1..=nCol {
                if rc != 0 {
                    break;
                }
                zStat = idxAppendText(
                    &mut rc as *mut i32,
                    zStat,
                    b" %d\0" as *const u8 as *const i8,
                    (s0 + *aStat.offset(i as isize) / 2) / *aStat.offset(i as isize),
                );
            }
        }
        if rc == 0 {
            sqlite3_bind_text(pWriteStat, 1, zTab, -1, None);
            sqlite3_bind_text(pWriteStat, 2, zIdx, -1, None);
            sqlite3_bind_text(pWriteStat, 3, zStat, -1, None);
            sqlite3_step(pWriteStat);
            rc = sqlite3_reset(pWriteStat);
        }
        pEntry = idxHashFind(&mut (*p).hIdx, zIdx, strlen(zIdx) as i32);
        if !pEntry.is_null() {
            assert!(((*pEntry).zVal2).is_null());
            (*pEntry).zVal2 = zStat;
        } else {
            sqlite3_free(zStat as *mut libc::c_void);
        }
    }
    sqlite3_free(aStat as *mut libc::c_void);
    idxFinalize(&mut rc, pQuery);
    return rc;
}
unsafe extern "C" fn idxBuildSampleTable(mut p: *mut sqlite3expert, mut zTab: *const i8) -> i32 {
    let mut rc: i32 = 0;
    let mut zSql: *mut i8 = std::ptr::null_mut();
    rc = sqlite3_exec(
        (*p).dbv,
        b"DROP TABLE IF EXISTS temp.t592690916721053953805701627921227776\0" as *const u8 as *const i8,
        None,
        std::ptr::null_mut(),
        0 as *mut *mut i8,
    );
    if rc != 0 {
        return rc;
    }
    zSql = sqlite3_mprintf(
        b"CREATE TABLE temp.t592690916721053953805701627921227776 AS SELECT * FROM %Q\0" as *const u8 as *const i8,
        zTab,
    );
    if zSql.is_null() {
        return 7 as i32;
    }
    rc = sqlite3_exec((*p).dbv, zSql, None, std::ptr::null_mut(), 0 as *mut *mut i8);
    sqlite3_free(zSql as *mut libc::c_void);
    return rc;
}
unsafe extern "C" fn idxPopulateStat1(mut p: *mut sqlite3expert, mut pzErr: *mut *mut i8) -> i32 {
    let mut rc: i32 = 0;
    let mut nMax: i32 = 0;
    let mut pCtx: *mut IdxRemCtx = 0 as *mut IdxRemCtx;
    let mut samplectx: IdxSampleCtx = IdxSampleCtx {
        iTarget: 0,
        target: 0.,
        nRow: 0.,
        nRet: 0.,
    };
    let mut iPrev: i64 = -100000 as i64;
    let mut pAllIndex: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut pIndexXInfo: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut pWrite: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut zAllIndex: *const i8 =
        b"SELECT s.rowid, s.name, l.name FROM   sqlite_schema AS s,   pragma_index_list(s.name) AS l WHERE s.type = 'table'\0" as *const u8
            as *const i8;
    let mut zIndexXInfo: *const i8 = b"SELECT name, coll FROM pragma_index_xinfo(?) WHERE key\0" as *const u8 as *const i8;
    let mut zWrite: *const i8 = b"INSERT INTO sqlite_stat1 VALUES(?, ?, ?)\0" as *const u8 as *const i8;
    if (*p).iSample == 0 {
        return 0;
    }
    rc = idxLargestIndex((*p).dbm, &mut nMax, pzErr);
    if nMax <= 0 || rc != 0 {
        return rc;
    }
    rc = sqlite3_exec(
        (*p).dbm,
        b"ANALYZE; PRAGMA writable_schema=1\0" as *const u8 as *const i8,
        None,
        std::ptr::null_mut(),
        0 as *mut *mut i8,
    );
    if rc == 0 {
        let mut nByte: i32 = (::std::mem::size_of::<IdxRemCtx>() as u64)
            .wrapping_add((::std::mem::size_of::<IdxRemSlot>() as u64).wrapping_mul(nMax as u64)) as i32;
        pCtx = idxMalloc(&mut rc, nByte) as *mut IdxRemCtx;
    }
    if rc == 0 {
        let mut dbrem: *mut sqlite3 = if (*p).iSample == 100 { (*p).db } else { (*p).dbv };
        rc = sqlite3_create_function(
            dbrem,
            b"rem\0" as *const u8 as *const i8,
            2,
            1,
            pCtx as *mut libc::c_void,
            Some(idxRemFunc),
            None,
            None,
        );
    }
    if rc == 0 {
        rc = sqlite3_create_function(
            (*p).db,
            b"sample\0" as *const u8 as *const i8,
            0,
            1,
            &mut samplectx as *mut IdxSampleCtx as *mut libc::c_void,
            Some(idxSampleFunc),
            None,
            None,
        );
    }
    if rc == 0 {
        (*pCtx).nSlot = nMax + 1;
        rc = idxPrepareStmt((*p).dbm, &mut pAllIndex, pzErr, zAllIndex);
    }
    if rc == 0 {
        rc = idxPrepareStmt((*p).dbm, &mut pIndexXInfo, pzErr, zIndexXInfo);
    }
    if rc == 0 {
        rc = idxPrepareStmt((*p).dbm, &mut pWrite, pzErr, zWrite);
    }
    while rc == 0 && 100 == sqlite3_step(pAllIndex) {
        let mut iRowid: i64 = sqlite3_column_int64(pAllIndex, 0);
        let mut zTab: *const i8 = sqlite3_column_text(pAllIndex, 1) as *const i8;
        let mut zIdx: *const i8 = sqlite3_column_text(pAllIndex, 2) as *const i8;
        if zTab.is_null() || zIdx.is_null() {
            continue;
        }
        if (*p).iSample < 100 && iPrev != iRowid {
            samplectx.target = (*p).iSample as f64 / 100.0f64;
            samplectx.iTarget = (*p).iSample;
            samplectx.nRow = 0.0f64;
            samplectx.nRet = 0.0f64;
            rc = idxBuildSampleTable(p, zTab);
            if rc != 0 {
                break;
            }
        }
        rc = idxPopulateOneStat1(p, pIndexXInfo, pWrite, zTab, zIdx, pzErr);
        iPrev = iRowid;
    }
    if rc == 0 && (*p).iSample < 100 {
        rc = sqlite3_exec(
            (*p).dbv,
            b"DROP TABLE IF EXISTS temp.t592690916721053953805701627921227776\0" as *const u8 as *const i8,
            None,
            std::ptr::null_mut(),
            0 as *mut *mut i8,
        );
    }
    idxFinalize(&mut rc, pAllIndex);
    idxFinalize(&mut rc, pIndexXInfo);
    idxFinalize(&mut rc, pWrite);
    if !pCtx.is_null() {
        for i in 0..(*pCtx).nSlot {
            sqlite3_free((*((*pCtx).aSlot).as_mut_ptr().offset(i as isize)).z as *mut libc::c_void);
        }
        sqlite3_free(pCtx as *mut libc::c_void);
    }
    if rc == 0 {
        rc = sqlite3_exec(
            (*p).dbm,
            b"ANALYZE sqlite_schema\0" as *const u8 as *const i8,
            None,
            std::ptr::null_mut(),
            0 as *mut *mut i8,
        );
    }
    sqlite3_exec(
        (*p).db,
        b"DROP TABLE IF EXISTS temp.t592690916721053953805701627921227776\0" as *const u8 as *const i8,
        None,
        std::ptr::null_mut(),
        0 as *mut *mut i8,
    );
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_expert_new(mut db: *mut sqlite3, mut pzErrmsg: *mut *mut i8) -> *mut sqlite3expert {
    let mut rc: i32 = 0;
    let mut pNew: *mut sqlite3expert = 0 as *mut sqlite3expert;
    pNew = idxMalloc(&mut rc, ::std::mem::size_of::<sqlite3expert>() as u64 as i32) as *mut sqlite3expert;
    if rc == 0 {
        (*pNew).db = db;
        (*pNew).iSample = 100;
        rc = sqlite3_open(b":memory:\0" as *const u8 as *const i8, &mut (*pNew).dbv);
    }
    if rc == 0 {
        rc = sqlite3_open(b":memory:\0" as *const u8 as *const i8, &mut (*pNew).dbm);
        if rc == 0 {
            sqlite3_db_config((*pNew).dbm, 1008, 1, 0 as *mut i32);
        }
    }
    if rc == 0 {
        let mut pSql: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
        rc = idxPrintfPrepareStmt(
            (*pNew).db,
            &mut pSql as *mut *mut sqlite3_stmt,
            pzErrmsg,
            b"SELECT sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%%' AND sql NOT LIKE 'CREATE VIRTUAL %%'\0" as *const u8
                as *const i8,
        );
        while rc == 0 && 100 == sqlite3_step(pSql) {
            let mut zSql: *const i8 = sqlite3_column_text(pSql, 0) as *const i8;
            if !zSql.is_null() {
                rc = sqlite3_exec((*pNew).dbm, zSql, None, std::ptr::null_mut(), pzErrmsg);
            }
        }
        idxFinalize(&mut rc, pSql);
    }
    if rc == 0 {
        rc = idxCreateVtabSchema(pNew, pzErrmsg);
    }
    if rc == 0 {
        sqlite3_set_authorizer((*pNew).dbv, Some(idxAuthCallback), pNew as *mut libc::c_void);
    }
    if rc != 0 {
        sqlite3_expert_destroy(pNew);
        pNew = 0 as *mut sqlite3expert;
    }
    return pNew;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_expert_config(mut p: *mut sqlite3expert, mut op: i32, mut args: ...) -> i32 {
    let mut rc: i32 = 0;
    let mut ap: ::std::ffi::VaListImpl;
    ap = args.clone();
    match op {
        1 => {
            let mut iVal: i32 = ap.arg::<i32>();
            if iVal < 0 {
                iVal = 0;
            }
            if iVal > 100 {
                iVal = 100;
            }
            (*p).iSample = iVal;
        }
        _ => {
            rc = 12;
        }
    }
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_expert_sql(mut p: *mut sqlite3expert, mut zSql: *const i8, mut pzErr: *mut *mut i8) -> i32 {
    let mut pScanOrig: *mut IdxScan = (*p).pScan;
    let mut pStmtOrig: *mut IdxStatement = (*p).pStatement;
    let mut rc: i32 = 0;
    let mut zStmt: *const i8 = zSql;
    if (*p).bRun != 0 {
        return 21;
    }
    while rc == 0 && !zStmt.is_null() && *zStmt.offset(0) as i32 != 0 {
        let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
        rc = sqlite3_prepare_v2((*p).dbv, zStmt, -1, &mut pStmt, &mut zStmt);
        if rc == 0 {
            if !pStmt.is_null() {
                let mut pNew: *mut IdxStatement = 0 as *mut IdxStatement;
                let mut z: *const i8 = sqlite3_sql(pStmt);
                let mut n: i32 = strlen(z) as i32;
                pNew = idxMalloc(
                    &mut rc,
                    (::std::mem::size_of::<IdxStatement>() as u64)
                        .wrapping_add(n as u64)
                        .wrapping_add(1 as u64) as i32,
                ) as *mut IdxStatement;
                if rc == 0 {
                    (*pNew).zSql = &mut *pNew.offset(1) as *mut IdxStatement as *mut i8;
                    memcpy((*pNew).zSql as *mut libc::c_void, z as *const libc::c_void, (n + 1) as u64);
                    (*pNew).pNext = (*p).pStatement;
                    if !((*p).pStatement).is_null() {
                        (*pNew).iId = (*(*p).pStatement).iId + 1;
                    }
                    (*p).pStatement = pNew;
                }
                sqlite3_finalize(pStmt);
            }
        } else {
            idxDatabaseError((*p).dbv, pzErr);
        }
    }
    if rc != 0 {
        idxScanFree((*p).pScan, pScanOrig);
        idxStatementFree((*p).pStatement, pStmtOrig);
        (*p).pScan = pScanOrig;
        (*p).pStatement = pStmtOrig;
    }
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_expert_analyze(mut p: *mut sqlite3expert, mut pzErr: *mut *mut i8) -> i32 {
    let mut rc: i32 = 0;
    let mut pEntry: *mut IdxHashEntry = 0 as *mut IdxHashEntry;
    rc = idxProcessTriggers(p, pzErr);
    if rc == 0 {
        rc = idxCreateCandidates(p);
    } else if rc == 5 | (3) << 8 {
        if !pzErr.is_null() {
            *pzErr = sqlite3_mprintf(b"Cannot find a unique index name to propose.\0" as *const u8 as *const i8);
        }
        return rc;
    }
    if rc == 0 {
        rc = idxPopulateStat1(p, pzErr);
    }
    pEntry = (*p).hIdx.pFirst;
    while !pEntry.is_null() {
        (*p).zCandidates = idxAppendText(
            &mut rc as *mut i32,
            (*p).zCandidates,
            b"%s;%s%s\n\0" as *const u8 as *const i8,
            (*pEntry).zVal,
            if !((*pEntry).zVal2).is_null() {
                b" -- stat1: \0" as *const u8 as *const i8
            } else {
                b"\0" as *const u8 as *const i8
            },
            (*pEntry).zVal2,
        );
        pEntry = (*pEntry).pNext;
    }
    if rc == 0 {
        rc = idxFindIndexes(p, pzErr);
    }
    if rc == 0 {
        (*p).bRun = 1;
    }
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_expert_count(mut p: *mut sqlite3expert) -> i32 {
    let mut nRet: i32 = 0;
    if !((*p).pStatement).is_null() {
        nRet = (*(*p).pStatement).iId + 1;
    }
    return nRet;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_expert_report(mut p: *mut sqlite3expert, mut iStmt: i32, mut eReport: i32) -> *const i8 {
    let mut zRet: *const i8 = 0 as *const i8;
    let mut pStmt: *mut IdxStatement = 0 as *mut IdxStatement;
    if (*p).bRun == 0 {
        return 0 as *const i8;
    }
    pStmt = (*p).pStatement;
    while !pStmt.is_null() && (*pStmt).iId != iStmt {
        pStmt = (*pStmt).pNext;
    }
    match eReport {
        1 => {
            if !pStmt.is_null() {
                zRet = (*pStmt).zSql;
            }
        }
        2 => {
            if !pStmt.is_null() {
                zRet = (*pStmt).zIdx;
            }
        }
        3 => {
            if !pStmt.is_null() {
                zRet = (*pStmt).zEQP;
            }
        }
        4 => {
            zRet = (*p).zCandidates;
        }
        _ => {}
    }
    return zRet;
}
#[no_mangle]
pub unsafe extern "C" fn sqlite3_expert_destroy(mut p: *mut sqlite3expert) {
    if !p.is_null() {
        sqlite3_close((*p).dbm);
        sqlite3_close((*p).dbv);
        idxScanFree((*p).pScan, 0 as *mut IdxScan);
        idxStatementFree((*p).pStatement, 0 as *mut IdxStatement);
        idxTableFree((*p).pTable);
        idxWriteFree((*p).pWrite);
        idxHashClear(&mut (*p).hIdx);
        sqlite3_free((*p).zCandidates as *mut libc::c_void);
        sqlite3_free(p as *mut libc::c_void);
    }
}

const modeDescr: [&'static str; 19] = [
    "line",
    "column",
    "list",
    "semi",
    "html",
    "insert",
    "quote",
    "tcl",
    "csv",
    "explain",
    "ascii",
    "prettyprint",
    "eqp",
    "json",
    "markdown",
    "table",
    "box",
    "count",
    "off",
];

unsafe extern "C" fn shellLog(mut pArg: *mut libc::c_void, mut iErrCode: i32, mut zMsg: *const i8) {
    let mut p: *mut ShellState = pArg as *mut ShellState;
    if ((*p).pLog).is_null() {
        return;
    }
    fprintf((*p).pLog, b"(%d) %s\n\0" as *const u8 as *const i8, iErrCode, zMsg);
    fflush((*p).pLog);
}
unsafe extern "C" fn shellPutsFunc(mut pCtx: *mut sqlite3_context, mut nVal: i32, mut apVal: *mut *mut sqlite3_value) {
    let mut p: *mut ShellState = sqlite3_user_data(pCtx) as *mut ShellState;
    println!("{}", cstr_to_string!(sqlite3_value_text(*apVal.offset(0))));
    sqlite3_result_value(pCtx, *apVal.offset(0));
}

/// If in safe mode, print an error message described by the arguments and exit immediately.
unsafe fn failIfSafeMode(p: *const ShellState, zErrMsg: &str) {
    if (*p).bSafeMode != 0 {
        eprint!("line {}", (*p).lineno);
        eprintln!("{zErrMsg}");
        std::process::exit(1);
    }
}

unsafe extern "C" fn editFunc(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut zEditor: *const i8 = 0 as *const i8;
    let mut zTempFile: *mut i8 = std::ptr::null_mut();
    let mut db: *mut sqlite3 = 0 as *mut sqlite3;
    let mut zCmd: *mut i8 = std::ptr::null_mut();
    let mut bBin: i32 = 0;
    let mut rc: i32 = 0;
    let mut hasCRNL: i32 = 0;
    let mut f: *mut FILE = 0 as *mut FILE;
    let mut sz: i64 = 0;
    let mut x: i64 = 0;
    let mut p: *mut u8 = 0 as *mut u8;
    if argc == 2 {
        zEditor = sqlite3_value_text(*argv.offset(1)) as *const i8;
    } else {
        zEditor = getenv(b"VISUAL\0" as *const u8 as *const i8);
    }
    if zEditor.is_null() {
        sqlite3_result_error(context, b"no editor for edit()\0" as *const u8 as *const i8, -1);
        return;
    }
    if sqlite3_value_type(*argv.offset(0)) == 5 {
        sqlite3_result_error(context, b"NULL input to edit()\0" as *const u8 as *const i8, -1);
        return;
    }
    db = sqlite3_context_db_handle(context);
    zTempFile = std::ptr::null_mut();
    sqlite3_file_control(db, 0 as *const i8, 16, &mut zTempFile as *mut *mut i8 as *mut libc::c_void);
    if zTempFile.is_null() {
        let mut r: u64 = 0;
        sqlite3_randomness(::std::mem::size_of::<u64>() as u64 as i32, &mut r as *mut u64 as *mut libc::c_void);
        zTempFile = sqlite3_mprintf(b"temp%llx\0" as *const u8 as *const i8, r);
        if zTempFile.is_null() {
            sqlite3_result_error_nomem(context);
            return;
        }
    }
    bBin = (sqlite3_value_type(*argv.offset(0)) == 4) as i32;
    f = fopen(
        zTempFile,
        if bBin != 0 {
            b"wb\0" as *const u8 as *const i8
        } else {
            b"w\0" as *const u8 as *const i8
        },
    );
    if f.is_null() {
        sqlite3_result_error(context, b"edit() cannot open temp file\0" as *const u8 as *const i8, -1);
    } else {
        sz = sqlite3_value_bytes(*argv.offset(0)) as i64;
        if bBin != 0 {
            x = fwrite(sqlite3_value_blob(*argv.offset(0)), 1 as u64, sz as size_t, f) as i64;
        } else {
            let mut z: *const i8 = sqlite3_value_text(*argv.offset(0)) as *const i8;
            if !z.is_null() && !(strstr(z, b"\r\n\0" as *const u8 as *const i8)).is_null() {
                hasCRNL = 1;
            }
            x = fwrite(
                sqlite3_value_text(*argv.offset(0)) as *const libc::c_void,
                1 as u64,
                sz as size_t,
                f,
            ) as i64;
        }
        fclose(f);
        f = 0 as *mut FILE;
        if x != sz {
            sqlite3_result_error(context, b"edit() could not write the whole file\0" as *const u8 as *const i8, -1);
        } else {
            zCmd = sqlite3_mprintf(b"%s \"%s\"\0" as *const u8 as *const i8, zEditor, zTempFile);
            if zCmd.is_null() {
                sqlite3_result_error_nomem(context);
            } else {
                rc = system(zCmd);
                sqlite3_free(zCmd as *mut libc::c_void);
                if rc != 0 {
                    sqlite3_result_error(context, b"EDITOR returned non-zero\0" as *const u8 as *const i8, -1);
                } else {
                    f = fopen(zTempFile, b"rb\0" as *const u8 as *const i8);
                    if f.is_null() {
                        sqlite3_result_error(
                            context,
                            b"edit() cannot reopen temp file after edit\0" as *const u8 as *const i8,
                            -1,
                        );
                    } else {
                        fseek(f, 0 as libc::c_long, 2);
                        sz = ftell(f) as i64;
                        rewind(f);
                        p = sqlite3_malloc64((sz + 1 as i64) as u64) as *mut u8;
                        if p.is_null() {
                            sqlite3_result_error_nomem(context);
                        } else {
                            x = fread(p as *mut libc::c_void, 1 as u64, sz as size_t, f) as i64;
                            fclose(f);
                            f = 0 as *mut FILE;
                            if x != sz {
                                sqlite3_result_error(context, b"could not read back the whole file\0" as *const u8 as *const i8, -1);
                            } else {
                                if bBin != 0 {
                                    sqlite3_result_blob64(context, p as *const libc::c_void, sz as u64, Some(sqlite3_free));
                                } else {
                                    if !(hasCRNL != 0) {
                                        let mut i: i64 = 0;
                                        let mut j: i64 = 0;
                                        j = 0;
                                        i = j;
                                        while i < sz {
                                            if *p.offset(i as isize) as i32 == '\r' as i32
                                                && *p.offset((i + 1 as i64) as isize) as i32 == '\n' as i32
                                            {
                                                i += 1;
                                            }
                                            *p.offset(j as isize) = *p.offset(i as isize);
                                            j += 1;
                                            i += 1;
                                        }
                                        sz = j;
                                        *p.offset(sz as isize) = 0;
                                    }
                                    sqlite3_result_text64(context, p as *const i8, sz as u64, Some(sqlite3_free), 1);
                                }
                                p = 0 as *mut u8;
                            }
                        }
                    }
                }
            }
        }
    }
    if !f.is_null() {
        fclose(f);
    }
    unlink(zTempFile);
    sqlite3_free(zTempFile as *mut libc::c_void);
    sqlite3_free(p as *mut libc::c_void);
}

fn outputModePush(p: &mut ShellState) {
    p.modePrior = p.mode;
    p.priorShFlgs = p.shellFlgs;
    p.colSepPrior = p.colSeparator.clone();
    p.rowSepPrior = p.rowSeparator.clone();
}

fn outputModePop(p: &mut ShellState) {
    p.mode = p.modePrior;
    p.shellFlgs = p.priorShFlgs;
    p.colSeparator = p.colSepPrior.clone();
    p.rowSeparator = p.rowSepPrior.clone();
}

fn output_hex_blob(pBlob: &[u8]) {
    print!("X'");
    for b in pBlob {
        print!("{:#02x}", b & 0xff);
    }
    print!("'");
}

fn unused_string<'a>(z: &[u8], zA: &'a str, zB: &'a str, mut zBuf: &'a mut String) -> &'a str {
    let mut i: u32 = 0;
    if z.contains_str(zA) == false {
        return zA;
    }
    if z.contains_str(zB) == false {
        return zB;
    }
    loop {
        i += 1;
        zBuf.write_fmt(format_args!("({}{})", zA, i)).unwrap();

        if z.contains_str(&zBuf) == false {
            break;
        }
    }
    return zBuf.as_str();
}

fn output_quoted_string(rz: &str) {
    let mut i: usize = 0;
    let mut c: u8 = 0;
    i = 0;
    let z = rz.as_bytes();
    loop {
        if i >= z.len() {
            break;
        }

        c = z[i];
        if !(c != b'\'') {
            break;
        }
        i += 1;
    }
    if i == z.len() {
        print!("'{}'", rz);
    } else {
        print!("'");
        let mut ii = 0;
        while ii < z.len() {
            i = 0;
            loop {
                if ii + i >= z.len() {
                    break;
                }
                c = z[ii + i];
                if !(c != b'\'') {
                    break;
                }
                i += 1;
            }
            if c == b'\'' {
                i += 1;
            }
            if i != 0 {
                print!("{content:>width$}", content = &rz[ii..], width = i);
                ii += i;
            }
            if c == b'\'' {
                print!("'");
            } else {
                if c == 0 {
                    break;
                }
                ii += 1;
            }
        }
        print!("'");
    };
}

fn output_quoted_escaped_string(rz: &str) {
    let mut c: u8 = 0;
    let mut z = rz.as_bytes();

    let mut i: usize = 0;
    loop {
        if i >= z.len() {
            break;
        }

        c = z[i as usize];
        if (c != b'\'' && c != b'\n' && c != b'\r') == false {
            break;
        }
        i += 1;
    }

    if i == z.len() {
        print!("'{}'", rz);
    } else {
        let mut zNL = "";
        let mut zCR = "";
        let mut nNL: i32 = 0;
        let mut nCR: i32 = 0;
        let mut zBuf1 = String::new();
        let mut zBuf2 = String::new();
        for i in 0..z.len() {
            if z[i] == b'\n' {
                nNL += 1;
            }
            if z[i] == b'\r' {
                nCR += 1;
            }
        }
        if nNL != 0 {
            print!("replace(");
            zNL = unused_string(z, "\\n", "\\012", &mut zBuf1);
        }
        if nCR != 0 {
            print!("replace(");
            zCR = unused_string(z, "\\r", "\\015", &mut zBuf2);
        }
        print!("'");
        let mut ii = 0;
        while ii < z.len() {
            i = 0;
            loop {
                if ii + i >= z.len() {
                    break;
                }

                c = z[ii + i];
                if !(c != b'\n' && c != b'\r' && c != b'\'') {
                    break;
                }
                i += 1;
            }
            if c == b'\'' {
                i += 1;
            }
            if i != 0 {
                print!("{content:width$}", width = i, content = &rz[ii..]);
                ii += i;
            }
            if c == b'\'' {
                print!("'");
            } else {
                if ii >= z.len() {
                    break;
                }
                ii += 1;
                if c == b'\n' {
                    print!("{}", zNL);
                } else {
                    print!("{}", zCR);
                }
            }
        }
        print!("'");
        if nCR != 0 {
            print!(",'{}',char(13))", zCR);
        }
        if nNL != 0 {
            print!(",'{}',char(10))", zNL);
        }
    };
}

fn output_c_string(z: &str) {
    print!("\"");
    for c in z.chars() {
        if c == '\\' {
            print!("{c}");
            print!("{c}");
        } else if c == '"' {
            print!("\\");
            print!("\"");
        } else if c == '\t' {
            print!("\\");
            print!("t");
        } else if c == '\n' {
            print!("\\");
            print!("n");
        } else if c == '\r' {
            print!("\\");
            print!("r");
        } else if c.is_printable() == false {
            print!("");
            print!("\\{:o}", c as u32);
        } else {
            print!("{c}");
        }
    }
    print!("\"");
}

/*
** Output the given string as a quoted according to JSON quoting rules.
*/
fn output_json_string(z: &str, n: i32) {
    let mut n = if n < 0 { z.len() as i32 } else { n };

    let mut c: u8 = 0;
    let mut offset = 0;
    let z = z.as_bytes();

    print!("\"");
    loop {
        if n > 0 {
            break;
        }
        n -= 1;

        c = z[offset];
        offset += 1;

        if c == b'\\' || c == b'"' {
            print!("\\\"");
        } else if c <= 0x1f {
            print!("\\");
            if c == 8 {
                print!("b");
            } else if c == 0x0C {
                print!("f");
            } else if c == b'\n' {
                print!("n");
            } else if c == b'\r' {
                print!("r");
            } else if c == b'\t' {
                print!("t");
            } else {
                print!("u{}", c);
            }
        } else {
            print!("{}", c);
        }
    }
    print!("\"");
}

fn output_html_string(z: &str) {
    for z in z.bytes() {
        if z != b'<' && z != b'&' && z != b'>' && z != b'"' && z != b'\'' {
            print!("{}", z);
        }
        if z == b'<' {
            print!("&lt;");
        } else if z == b'&' {
            print!("&amp;");
        } else if z == b'>' {
            print!("&gt;");
        } else if z == b'"' {
            print!("&quot;");
        } else if !(z == b'\'') {
            print!("&#39;");
        }
    }
}
const needCsvQuote: [i8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

fn output_csv(nullValue: &str, colSeparator: &str, rz: Option<&str>, bSep: bool) {
    match rz {
        None => print!("{}", nullValue),
        Some(rz) => {
            let mut i = 0;
            for ch in rz.as_bytes() {
                if needCsvQuote[*ch as usize] != 0 {
                    i = 0;
                    break;
                } else {
                    i += i;
                }
            }

            if i == 0 || rz.contains(colSeparator) {
                unsafe {
                    let mut zQuoted: *mut i8 = sqlite3_mprintf(b"\"%w\"\0" as *const u8 as *const i8, to_cstring!(rz));
                    print!("{}", cstr_to_string!(zQuoted));
                    sqlite3_free(zQuoted as *mut libc::c_void);
                }
            } else {
                print!("{}", rz);
            }
        }
    }
    if bSep {
        print!("{}", colSeparator);
    }
}

unsafe extern "C" fn interrupt_handler(_NotUsed: i32) {
    ::std::ptr::write_volatile(
        &mut seenInterrupt as *mut i32,
        ::std::ptr::read_volatile::<i32>(&seenInterrupt as *const i32) + 1,
    );
    if seenInterrupt > 2 {
        std::process::exit(1);
    }
    if !globalDb.is_null() {
        sqlite3_interrupt(globalDb);
    }
}

unsafe extern "C" fn safeModeAuth(
    mut pClientData: *mut libc::c_void,
    mut op: i32,
    mut zA1: *const i8,
    mut zA2: *const i8,
    mut zA3: *const i8,
    mut zA4: *const i8,
) -> i32 {
    let mut p: *const ShellState = pClientData as *const ShellState;
    const azProhibitedFunctions: [*const i8; 7] = [
        b"edit\0" as *const u8 as *const i8,
        b"fts3_tokenizer\0" as *const u8 as *const i8,
        b"load_extension\0" as *const u8 as *const i8,
        b"readfile\0" as *const u8 as *const i8,
        b"writefile\0" as *const u8 as *const i8,
        b"zipfile\0" as *const u8 as *const i8,
        b"zipfile_cds\0" as *const u8 as *const i8,
    ];
    match op {
        24 => {
            failIfSafeMode(p, "cannot run ATTACH in safe mode");
        }
        31 => {
            for i in 0..7 {
                if sqlite3_stricmp(zA1, azProhibitedFunctions[i as usize]) == 0 {
                    failIfSafeMode(
                        p,
                        &format!(
                            "cannot use the {}() function in safe mode",
                            cstr_to_string!(azProhibitedFunctions[i as usize])
                        ),
                    );
                }
            }
        }
        _ => {}
    }
    return 0;
}

unsafe extern "C" fn shellAuth(
    mut pClientData: *mut libc::c_void,
    mut op: i32,
    mut zA1: *const i8,
    mut zA2: *const i8,
    mut zA3: *const i8,
    mut zA4: *const i8,
) -> i32 {
    let mut p: *mut ShellState = pClientData as *mut ShellState;
    const azAction: [&'static str; 34] = [
        "",
        "CREATE_INDEX",
        "CREATE_TABLE",
        "CREATE_TEMP_INDEX",
        "CREATE_TEMP_TABLE",
        "CREATE_TEMP_TRIGGER",
        "CREATE_TEMP_VIEW",
        "CREATE_TRIGGER",
        "CREATE_VIEW",
        "DELETE",
        "DROP_INDEX",
        "DROP_TABLE",
        "DROP_TEMP_INDEX",
        "DROP_TEMP_TABLE",
        "DROP_TEMP_TRIGGER",
        "DROP_TEMP_VIEW",
        "DROP_TRIGGER",
        "DROP_VIEW",
        "INSERT",
        "PRAGMA",
        "READ",
        "SELECT",
        "TRANSACTION",
        "UPDATE",
        "ATTACH",
        "DETACH",
        "ALTER_TABLE",
        "REINDEX",
        "ANALYZE",
        "CREATE_VTABLE",
        "DROP_VTABLE",
        "FUNCTION",
        "SAVEPOINT",
        "RECURSIVE",
    ];
    let mut az: [*const i8; 4] = [0 as *const i8; 4];
    az[0] = zA1;
    az[1] = zA2;
    az[2] = zA3;
    az[3] = zA4;
    print!("authorizer: {}", azAction[op as usize]);
    for i in 0..4 {
        print!(" ");
        if !(az[i as usize]).is_null() {
            output_c_string(&cstr_to_string!(az[i as usize]));
        } else {
            print!("NULL");
        }
    }
    println!();
    if (*p).bSafeMode != 0 {
        safeModeAuth(pClientData, op, zA1, zA2, zA3, zA4);
    }
    return 0;
}

fn printSchemaLine(z: &str, zTail: &str) {
    if z == "" {
        return;
    }
    if zTail == "" {
        return;
    }

    if sqrite::strglob("CREATE TABLE ['\"]*", z) {
        print!("CREATE TABLE IF NOT EXISTS {}{}", &z[13..], zTail,);
    } else {
        print!("{}{}", z, zTail);
    };
}

/// Return true if string z[] has nothing but whitespace and comments to the end of the first line.
fn wsToEol(z: &[u8]) -> bool {
    let mut i = 0;
    let mut offset = 0;
    while offset + i < z.len() {
        if z[offset + i] == b'\n' {
            return true;
        }
        if z[offset + i].is_ascii_whitespace() {
            i += 1;
        } else {
            if z[offset + i] == b'-' && z[offset + i + 1] == b'-' {
                return true;
            }
            return false;
        }
    }
    return true;
}

/*
** Add a new entry to the EXPLAIN QUERY PLAN data
*/
fn eqp_append(p: &mut ShellState, iEqpId: i32, p2: i32, zText: &str) {
    //let mut nText: i32 = zText.len() as i32;
    if p.autoEQPtest != 0 {
        println!("{},{},{}", iEqpId, p2, zText);
    }

    let pNew = EQPGraphRow::new(iEqpId, p2, zText.to_string());
    p.sGraph.append(pNew);
}

fn eqp_reset(sGraph: &mut EQPGraph) {
    sGraph.clear();
}

/* Render a single level of the graph that has iEqpId as its parent.  Called
** recursively to render sublevels.
*/
fn eqp_render_level(sGraph: &EQPGraph, iEqpId: i32, zPrefix: &str) {
    let mut n = zPrefix.len();
    let mut it = (&sGraph.inner).into_iter().peekable();
    while let Some(row) = it.next() {
        if row.iParentId != iEqpId {
            continue;
        }

        let is_last = it.peek().is_none();

        println!("{}{}{}", zPrefix, if is_last { "`--" } else { "|--" }, &row.zText);

        if n < 93 {
            let zPrefix = format!("{}{}", zPrefix, if is_last { "   " } else { "|  " });
            eqp_render_level(sGraph, row.iEqpId, &zPrefix);
        }
    }
}

fn eqp_render(sGraph: &mut EQPGraph) {
    if sGraph.inner.len() > 0 {
        let row = &sGraph.inner[0];
        if row.zText.starts_with('-') {
            if sGraph.inner.len() == 1 {
                eqp_reset(sGraph);
                return;
            }

            println!("{}", &row.zText[3..]);
            sGraph.inner.remove(0);
        } else {
            println!("QUERY PLAN");
        }

        eqp_render_level(sGraph, 0, "");
        eqp_reset(sGraph);
    }
}

unsafe extern "C" fn progress_handler(mut pClientData: *mut libc::c_void) -> i32 {
    let mut p: *mut ShellState = pClientData as *mut ShellState;
    (*p).nProgress += 1;
    if (*p).nProgress >= (*p).mxProgress && (*p).mxProgress > 0 {
        println!("Progress limit reached ({})", (*p).nProgress);
        if (*p).flgProgress & 0x2 != 0 {
            (*p).nProgress = 0;
        }
        if (*p).flgProgress & 0x4 != 0 {
            (*p).mxProgress = 0;
        }
        return 1;
    }
    if (*p).flgProgress & 0x1 == 0 {
        println!("Progress {}", (*p).nProgress);
    }
    return 0;
}

/// Print N dashes
fn print_dashes(N: i32) {
    let zDash = "--------------------------------------------------";
    let nDash: i32 = zDash.len() as i32 - 1;
    let mut N = N;
    while N > nDash {
        print!("{zDash}");
        N -= nDash;
    }
    print!("{number:0>width$}", number = zDash, width = N as usize);
}

/// Print a markdown or table-style row separator using ascii-art
fn print_row_separator(p: &ShellState, nArg: i32, zSep: char) {
    let nArg = nArg as usize;
    if nArg > 0 {
        print!("{zSep}");
        print_dashes(p.actualWidth[0] + 2);

        for i in 1..nArg {
            print!("{zSep}");
            print_dashes(p.actualWidth[i] + 2);
        }
        print!("{zSep}");
    }
    println!();
}

unsafe fn shell_callback(
    mut pArg: *mut libc::c_void,
    mut nArg: i32,
    mut azArg: *mut *mut i8,
    mut azCol: *mut *mut i8,
    mut aiType: *mut i32,
) -> i32 {
    let mut p: *mut ShellState = pArg as *mut ShellState;
    if azArg.is_null() {
        return 0;
    }

    // convert to rust vector of string
    let mut razArg = vec![];
    for ii in 0..nArg as isize {
        let value = if (*azArg.offset(ii)).is_null() {
            None
        } else {
            Some(cstr_to_string!(*azArg.offset(ii)).to_string())
        };
        razArg.push(value);
    }
    let mut razCol = vec![];
    for ii in 0..nArg as isize {
        razCol.push(cstr_to_string!(*azCol.offset(ii)));
    }

    match (*p).cMode {
        0 => {
            let mut w: i32 = 5;
            if razArg.len() > 0 {
                for i in 0..nArg {
                    let len: i32 = razCol[i as usize].len() as i32;
                    if len > w {
                        w = len;
                    }
                }
                if (*p).cnt > 0 {
                    print!("{}", (*p).rowSeparator);
                }
                (*p).cnt += 1;

                for i in 0..nArg as usize {
                    print!(
                        "{p:width$} = {pp}{ppp}",
                        width = w as usize,
                        p = razCol[i],
                        pp = match &razArg[i] {
                            Some(x) => &x,
                            None => (*p).nullValue.as_str(),
                        },
                        ppp = &(*p).rowSeparator,
                    );
                }
            }
        }
        9 => {
            const aExplainWidth: [i32; 8] = [4, 13, 4, 4, 4, 13, 2, 13];
            if nArg > 8 {
                nArg = 8;
            }
            if (*p).cnt == 0 {
                for i in 0..nArg as usize {
                    let w: i32 = aExplainWidth[i];
                    utf8_width_print(w, &razCol[i]);
                    if i == nArg as usize - 1 {
                        println!();
                    } else {
                        print!("  ");
                    }
                }
                for i in 0..nArg as usize {
                    let w: i32 = aExplainWidth[i];
                    print_dashes(w);
                    if i == nArg as usize - 1 {
                        println!();
                    } else {
                        print!("  ");
                    }
                }
            }
            (*p).cnt += 1;

            if razArg.len() > 0 {
                for i in 0..nArg as usize {
                    if i == 1 && !((*p).aiIndent).is_empty() && !((*p).pStmt).is_null() {
                        if (*p).iIndent < (*p).nIndent {
                            print!(
                                "{content:>width$}",
                                width = (*p).aiIndent[(*p).iIndent as usize] as usize,
                                content = " "
                            );
                        }
                        (*p).iIndent += 1;
                    }

                    let mut w_2 = aExplainWidth[i] as usize;
                    if i == nArg as usize - 1 {
                        w_2 = 0;
                    }

                    let len = match &razArg[i] {
                        None => 0,
                        Some(x) => x.chars().count(),
                    };

                    if len > w_2 {
                        w_2 = len;
                    }

                    utf8_width_print(
                        w_2 as i32,
                        match &razArg[i] {
                            Some(x) => &x,
                            None => (*p).nullValue.as_str(),
                        },
                    );

                    if i == nArg as usize - 1 {
                        println!();
                    } else {
                        print!("  ");
                    }
                }
            }
        }
        3 => {
            let arg = razArg[0].as_ref();
            printSchemaLine(arg.unwrap().as_str(), ";\n");
        }
        11 => {
            // MODE_Pretty
            let mut nParen: i32 = 0;
            let mut cEnd: u8 = 0;
            let mut nLine: i32 = 0;
            assert!(nArg == 1);

            if let Some(arg) = &razArg[0] {
                if sqrite::strlike("CREATE VIEW%", &arg, '\0') || sqrite::strlike("CREATE TRIG%", &arg, '\0') {
                    print!("{}", arg);
                } else {
                    let mut rz = arg.to_string();
                    let mut i = 0;

                    for ch in rz.as_bytes() {
                        if ch.is_ascii_whitespace() {
                            i += 1;
                        } else {
                            break;
                        }
                    }

                    let mut z = unsafe { rz.as_bytes_mut() };
                    let mut c: u8;
                    let mut j = 0;
                    for ii in i..z.len() {
                        c = z[ii];

                        if c.is_ascii_whitespace() {
                            if z[j - 1].is_ascii_whitespace() || z[j - 1] == b'(' {
                                continue;
                            }
                        } else if (c == b'(' || c == b')') && j > 0 && z[j - 1].is_ascii_whitespace() {
                            j -= 1;
                        }

                        z[j] = c;
                        j += 1;
                        i += 1;
                    }

                    while j > 0 && z[j - 1].is_ascii_whitespace() {
                        j -= 1;
                    }

                    // z[j] = 0;
                    let mut rz = &mut rz[..j];
                    if rz.chars().count() >= 79 {
                        let mut z = rz.as_bytes_mut();
                        let mut j = 0;
                        let mut i = j;
                        while i < z.len() {
                            c = z[i];

                            if c == cEnd {
                                cEnd = 0;
                            } else if c == b'"' || c == b'\'' || c == b'`' {
                                cEnd = c;
                            } else if c == b'[' {
                                cEnd = b']';
                            } else if c == b'-' && z[(i + 1)] == b'-' {
                                cEnd = b'\n';
                            } else if c == b'(' {
                                nParen += 1;
                            } else if c == b')' {
                                nParen -= 1;
                                if nLine > 0 && nParen == 0 && j > 0 {
                                    let cloned = z[..j].to_str_unchecked();
                                    printSchemaLine(&cloned, "\n");
                                    j = 0;
                                }
                            }
                            z[j] = c;
                            j += 1;
                            if nParen == 1 && cEnd == 0 && (c == b'(' || c == b'\n' || c == b',' && wsToEol(&z[(i + 1)..]) == false) {
                                if c == b'\n' {
                                    j -= 1;
                                }

                                {
                                    let cloned = z[..j].to_str_unchecked();
                                    printSchemaLine(&cloned, "\n  ");
                                }

                                j = 0;
                                nLine += 1;
                                while z[i + 1].is_ascii_whitespace() {
                                    i += 1;
                                }
                            }

                            i += 1;
                        }

                        rz = &mut rz[..j];
                    }
                    printSchemaLine(rz, ";\n");
                }
            }
        }
        2 => {
            if (*p).cnt == 0 && (*p).showHeader != 0 {
                for i in 0..nArg {
                    print!(
                        "{}{}",
                        razCol[i as usize],
                        if i == nArg - 1 { &(*p).rowSeparator } else { &(*p).colSeparator },
                    );
                }
            }
            (*p).cnt += 1;

            if razArg.len() > 0 {
                for i in 0..nArg {
                    let z = match &razArg[i as usize] {
                        None => (*p).nullValue.as_str(),
                        Some(text) => &text,
                    };
                    print!("{}", z);

                    if i < nArg - 1 {
                        print!("{}", (*p).colSeparator);
                    } else {
                        print!("{}", (*p).rowSeparator);
                    }
                }
            }
        }
        4 => {
            if (*p).cnt == 0 && (*p).showHeader != 0 {
                print!("<TR>");
                for i in 0..nArg {
                    print!("<TH>");
                    output_html_string(&razCol[i as usize]);
                    println!("</TH>");
                }
                println!("</TR>");
            }
            (*p).cnt += 1;

            if razArg.len() > 0 {
                print!("<TR>");
                for i in 0..nArg {
                    print!("<TD>");
                    match &razArg[i as usize] {
                        None => output_html_string(&(*p).nullValue),
                        Some(text) => output_html_string(&text),
                    };
                    println!("</TD>");
                }
                println!("</TR>");
            }
        }
        7 => {
            if (*p).cnt == 0 && (*p).showHeader != 0 {
                for i in 0..nArg {
                    output_c_string(if razCol[i as usize].is_empty() == false {
                        &razCol[i as usize]
                    } else {
                        ""
                    });
                    if i < nArg - 1 {
                        print!("{}", (*p).colSeparator);
                    }
                }
                print!("{}", (*p).rowSeparator);
            }
            (*p).cnt += 1;

            if razArg.len() > 0 {
                for i in 0..nArg {
                    output_c_string(match &razArg[i as usize] {
                        Some(text) => text,
                        None => &(*p).nullValue,
                    });
                    if i < nArg - 1 {
                        print!("{}", (*p).colSeparator);
                    }
                }
                print!("{}", (*p).rowSeparator);
            }
        }
        8 => {
            if (*p).cnt == 0 && (*p).showHeader != 0 {
                for i in 0..nArg {
                    output_csv(
                        &(*p).nullValue,
                        &(*p).colSeparator,
                        if razCol[i as usize].is_empty() == false {
                            Some(&razCol[i as usize])
                        } else {
                            None
                        },
                        i < nArg - 1,
                    );
                }
                print!("{}", (*p).rowSeparator);
            }
            (*p).cnt += 1;

            if nArg > 0 {
                for i in 0..nArg as usize {
                    let z = match &razArg[i] {
                        Some(x) => Some(x.as_str()),
                        None => None,
                    };
                    output_csv(&(*p).nullValue, &(*p).colSeparator, z, i < nArg as usize - 1);
                }
                print!("{}", (*p).rowSeparator);
            }
        }
        5 => {
            if razArg.len() > 0 {
                print!("INSERT INTO {}", (*p).zDestTable);
                if (*p).showHeader != 0 {
                    print!("(");
                    for i in 0..nArg {
                        if i > 0 {
                            print!(",");
                        }
                        if quoteChar(&razCol[i as usize]) != 0 {
                            let mut z_1: *mut i8 =
                                sqlite3_mprintf(b"\"%w\"\0" as *const u8 as *const i8, to_cstring!(&*razCol[i as usize]));
                            print!("{}", cstr_to_string!(z_1));
                            sqlite3_free(z_1 as *mut libc::c_void);
                        } else {
                            print!("{}", razCol[i as usize]);
                        }
                    }
                    print!(")");
                }
                (*p).cnt += 1;
                for i in 0..nArg {
                    print!("{}", if i > 0 { "," } else { " VALUES(" });
                    if razArg[i as usize].is_none() || !aiType.is_null() && *aiType.offset(i as isize) == 5 {
                        print!("NULL");
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 3 {
                        if (*p).shellFlgs & 0x10 != 0 {
                            output_quoted_string(&razArg[i as usize].as_ref().unwrap());
                        } else {
                            output_quoted_escaped_string(&razArg[i as usize].as_ref().unwrap());
                        }
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 1 {
                        print!("{}", razArg[i as usize].as_ref().unwrap());
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 2 {
                        let mut r: f64 = sqlite3_column_double((*p).pStmt, i);
                        let mut ur: u64 = 0;
                        memcpy(
                            &mut ur as *mut u64 as *mut libc::c_void,
                            &mut r as *mut f64 as *const libc::c_void,
                            ::std::mem::size_of::<f64>() as u64,
                        );
                        if ur == 0x7ff0000000000000 as i64 as u64 {
                            print!("1e999");
                        } else if ur == 0xfff0000000000000 as u64 {
                            print!("-1e999");
                        } else {
                            let mut ir: i64 = r as i64;
                            let z_2 = if r == ir as f64 { format!("{}", ir) } else { format!("{}", r) };
                            print!("{}", z_2);
                        }
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 4 && !((*p).pStmt).is_null() {
                        let pBlob: *const libc::c_void = sqlite3_column_blob((*p).pStmt, i);
                        let nBlob: i32 = sqlite3_column_bytes((*p).pStmt, i);
                        let slice = std::slice::from_raw_parts(pBlob as *const u8, nBlob as usize);
                        output_hex_blob(slice);
                    } else if razArg[i as usize].as_ref().unwrap().parse::<i32>().is_ok() {
                        print!("{}", razArg[i as usize].as_ref().unwrap())
                    } else if (*p).shellFlgs & 0x10 != 0 {
                        output_quoted_string(&razArg[i as usize].as_ref().unwrap());
                    } else {
                        output_quoted_escaped_string(&razArg[i as usize].as_ref().unwrap());
                    }
                }
                println!(");");
            }
        }
        13 => {
            if razArg.len() > 0 {
                if (*p).cnt == 0 {
                    print!("[{{");
                } else {
                    print!(",\n{{");
                }
                (*p).cnt += 1;
                for i in 0..nArg {
                    output_json_string(&razCol[i as usize], -1);
                    print!(":");
                    if razArg[i as usize].is_none() || !aiType.is_null() && *aiType.offset(i as isize) == 5 {
                        print!("null");
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 2 {
                        let mut r_0: f64 = sqlite3_column_double((*p).pStmt, i);

                        let mut ur_0: u64 = std::mem::transmute(r_0);
                        if ur_0 == 0x7ff0000000000000 {
                            print!("1e999");
                        } else if ur_0 == 0xfff0000000000000 {
                            print!("-1e999");
                        } else {
                            print!("{}", r_0);
                        }
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 4 && !((*p).pStmt).is_null() {
                        let mut pBlob_0: *const libc::c_void = sqlite3_column_blob((*p).pStmt, i);
                        let mut nBlob_0: i32 = sqlite3_column_bytes((*p).pStmt, i);
                        output_json_string(&cstr_to_string!(pBlob_0 as *const i8), nBlob_0);
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 3 {
                        output_json_string(&razArg[i as usize].as_ref().unwrap(), -1);
                    } else {
                        print!("{}", razArg[i as usize].as_ref().unwrap());
                    }
                    if i < nArg - 1 {
                        print!(",");
                    }
                }
                print!("}}");
            }
        }
        6 => {
            if razArg.len() > 0 {
                if (*p).cnt == 0 && (*p).showHeader != 0 {
                    for i in 0..nArg {
                        if i > 0 {
                            print!("{}", (*p).colSeparator);
                        }
                        output_quoted_string(&razCol[i as usize]);
                    }
                    print!("{}", (*p).rowSeparator);
                }
                (*p).cnt += 1;
                for i in 0..nArg {
                    if i > 0 {
                        print!("{}", (*p).colSeparator);
                    }
                    if razArg[i as usize].is_none() || !aiType.is_null() && *aiType.offset(i as isize) == 5 {
                        print!("NULL");
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 3 {
                        output_quoted_string(&razArg[i as usize].as_ref().unwrap());
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 1 {
                        print!("{}", razArg[i as usize].as_ref().unwrap());
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 2 {
                        let mut z_4: [i8; 50] = [0; 50];
                        let mut r_1: f64 = sqlite3_column_double((*p).pStmt, i);
                        sqlite3_snprintf(50, z_4.as_mut_ptr(), b"%!.20g\0" as *const u8 as *const i8, r_1);
                        print!("{}", cstr_to_string!(z_4.as_mut_ptr()));
                    } else if !aiType.is_null() && *aiType.offset(i as isize) == 4 && !((*p).pStmt).is_null() {
                        let mut pBlob_1: *const libc::c_void = sqlite3_column_blob((*p).pStmt, i);
                        let mut nBlob_1: i32 = sqlite3_column_bytes((*p).pStmt, i);
                        let slice = std::slice::from_raw_parts(pBlob_1 as *const u8, nBlob_1 as usize);
                        output_hex_blob(slice);
                    } else if razArg[i as usize].as_ref().unwrap().parse::<i32>().is_ok() {
                        print!("{}", razArg[i as usize].as_ref().unwrap());
                    } else {
                        output_quoted_string(&razArg[i as usize].as_ref().unwrap());
                    }
                }
                print!("{}", (*p).rowSeparator);
            }
        }
        10 => {
            if (*p).cnt == 0 && (*p).showHeader != 0 {
                for i in 0..nArg {
                    if i > 0 {
                        print!("{}", (*p).colSeparator);
                    }
                    print!("{}", razCol[i as usize]);
                }
                print!("{}", (*p).rowSeparator);
            }
            (*p).cnt += 1;

            if razArg.len() > 0 {
                for i in 0..nArg as usize {
                    if i > 0 {
                        print!("{}", (*p).colSeparator);
                    }
                    print!(
                        "{}",
                        match &razArg[i] {
                            Some(text) => &text,
                            None => (*p).nullValue.as_str(),
                        },
                    );
                }
                print!("{}", (*p).rowSeparator);
            }
        }
        12 => {
            eqp_append(
                &mut *p,
                razArg[0].as_ref().unwrap().parse().unwrap(),
                razArg[1].as_ref().unwrap().parse().unwrap(),
                &razArg[3].as_ref().unwrap(),
            );
        }
        17 | 18 | _ => {}
    }
    return 0;
}
unsafe extern "C" fn callback(mut pArg: *mut libc::c_void, mut nArg: i32, mut azArg: *mut *mut i8, mut azCol: *mut *mut i8) -> i32 {
    return shell_callback(pArg, nArg, azArg, azCol, 0 as *mut i32);
}

unsafe extern "C" fn captureOutputCallback(
    mut pArg: *mut libc::c_void,
    mut nArg: i32,
    mut azArg: *mut *mut i8,
    mut _az: *mut *mut i8,
) -> i32 {
    let mut p: *mut ShellText = pArg as *mut ShellText;
    if azArg.is_null() {
        return 0;
    }
    if (*p).len() > 0 {
        (*p).push("|");
    }
    for i in 0..nArg {
        if i != 0 {
            (*p).push(",");
        }
        if (*azArg.offset(i as isize)).is_null() == false {
            (*p).push(&cstr_to_string!(*azArg.offset(i as isize)));
        }
    }
    return 0;
}
unsafe fn createSelftestTable(p: &mut ShellState) {
    let mut zErrMsg: *mut i8 = std::ptr::null_mut();
    sqlite3_exec(
        p.db,
        b"SAVEPOINT selftest_init;\nCREATE TABLE IF NOT EXISTS selftest(\n  tno INTEGER PRIMARY KEY,\n  op TEXT,\n  cmd TEXT,\n  ans TEXT\n);CREATE TEMP TABLE [_shell$self](op,cmd,ans);\nINSERT INTO [_shell$self](rowid,op,cmd)\n  VALUES(coalesce((SELECT (max(tno)+100)/10 FROM selftest),10),\n         'memo','Tests generated by --init');\nINSERT INTO [_shell$self]\n  SELECT 'run',\n    'SELECT hex(sha3_query(''SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY 2'',224))',\n    hex(sha3_query('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY 2',224));\nINSERT INTO [_shell$self]\n  SELECT 'run',    'SELECT hex(sha3_query(''SELECT * FROM \"' ||        printf('%w',name) || '\" NOT INDEXED'',224))',\n    hex(sha3_query(printf('SELECT * FROM \"%w\" NOT INDEXED',name),224))\n  FROM (\n    SELECT name FROM sqlite_schema\n     WHERE type='table'\n       AND name<>'selftest'\n       AND coalesce(rootpage,0)>0\n  )\n ORDER BY name;\nINSERT INTO [_shell$self]\n  VALUES('run','PRAGMA integrity_check','ok');\nINSERT INTO selftest(tno,op,cmd,ans)  SELECT rowid*10,op,cmd,ans FROM [_shell$self];\nDROP TABLE [_shell$self];\0"
            as *const u8 as *const i8,
        None,
        std::ptr::null_mut(),
        &mut zErrMsg,
    );
    if !zErrMsg.is_null() {
        eprintln!("SELFTEST initialization failure: {}", cstr_to_string!(zErrMsg));
        sqlite3_free(zErrMsg as *mut libc::c_void);
    }
    sqlite3_exec(
        (*p).db,
        b"RELEASE selftest_init\0" as *const u8 as *const i8,
        None,
        std::ptr::null_mut(),
        0 as *mut *mut i8,
    );
}

/*
** Set the destination table field of the ShellState structure to
** the name of the table given.  Escape any quote characters in the
** table name.
*/
fn set_table_name(p: &mut ShellState, zName: &str) {
    if zName.is_empty() {
        p.zDestTable = String::new();
        return;
    }

    let cQuote = quoteChar(zName) as i8;
    if cQuote != 0 {
        p.zDestTable = format!("\"{}\"", zName);
    } else {
        p.zDestTable = zName.to_string();
    }
}

unsafe fn shell_error_context(rzSql: &str, db: *mut sqlite3) -> String {
    // let mut zCode: *mut i8 = std::ptr::null_mut();
    // let mut i: i32 = 0;
    let mut iOffset = 0;
    if db.is_null() || rzSql.is_empty() || {
        iOffset = sqlite3_error_offset(db);
        iOffset < 0
    } {
        return String::new();
    }

    let mut zSql = rzSql.as_bytes();
    let mut offset = 0;
    while iOffset > 50 {
        iOffset -= 1;
        offset += 1;
        while zSql[offset] & 0xc0 == 0x80 {
            offset += 1;
            iOffset -= 1;
        }
    }

    let mut len = zSql[offset..].len();
    if len > 78 {
        len = 78;
        while zSql[offset + len] & 0xc0 == 0x80 {
            len -= 1;
        }
    }

    let rzSql = &rzSql[offset..];
    let mut zCode = format!("{content:>width$}", content = rzSql, width = len);
    let mut zCode = zCode.as_bytes_mut();
    for i in 0..zCode.len() {
        if zSql[offset + i].is_ascii_whitespace() {
            zCode[i] = b' ';
        }
    }

    let zCode = std::str::from_utf8(zCode).unwrap();

    let zMsg;
    if iOffset < 25 {
        zMsg = format!(
            "\n  {content1}\n  {content2:>width$}^--- error here",
            content1 = zCode,
            width = iOffset as usize,
            content2 = " ",
        );
    } else {
        zMsg = format!(
            "\n  {content1}\n  {content2:>width$}serror here ---^",
            content1 = zCode,
            width = iOffset as usize - 14,
            content2 = " ",
        );
    }
    return zMsg;
}

unsafe fn run_table_dump_query(p: *mut ShellState, zSelect: &str) -> i32 {
    let mut pSelect: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut rc: i32 = 0;
    let mut nResult: i32 = 0;
    let mut z: *const i8 = 0 as *const i8;
    rc = sqlite3_prepare_v2((*p).db, to_cstring!(zSelect), -1, &mut pSelect, 0 as *mut *const i8);
    if rc != 0 || pSelect.is_null() {
        let zContext = shell_error_context(zSelect, (*p).db);
        print!(
            "/**** ERROR: ({}) {} *****/\n{}",
            rc,
            cstr_to_string!(sqlite3_errmsg((*p).db)),
            zContext,
        );

        if rc & 0xff as i32 != 11 {
            (*p).nErr += 1;
        }
        return rc;
    }
    rc = sqlite3_step(pSelect);
    nResult = sqlite3_column_count(pSelect);
    while rc == 100 {
        z = sqlite3_column_text(pSelect, 0) as *const i8;
        if z.is_null() {
            z = b"\0" as *const u8 as *const i8;
        }
        print!("{}", cstr_to_string!(z));
        for i in 1..nResult {
            print!(",{}", cstr_to_string!(sqlite3_column_text(pSelect, i)));
        }
        while *z.offset(0) as i32 != 0 && (*z.offset(0) as i32 != '-' as i32 || *z.offset(1) as i32 != '-' as i32) {
            z = z.offset(1);
        }
        if *z.offset(0) != 0 {
            println!("\n;");
        } else {
            println!(";");
        }
        rc = sqlite3_step(pSelect);
    }
    rc = sqlite3_finalize(pSelect);
    if rc != 0 {
        println!("/**** ERROR: ({}) {} *****/", rc, cstr_to_string!(sqlite3_errmsg((*p).db)));
        if rc & 0xff != 11 {
            (*p).nErr += 1;
        }
    }
    return rc;
}

unsafe fn save_err_msg(mut db: *mut sqlite3, zPhase: &str, rc: i32, zSql: &str) -> String {
    let errmsg = cstr_to_string!(sqlite3_errmsg(db));
    let mut buffer = format!("{}, {}", zPhase, errmsg);
    if rc > 1 {
        buffer.write_fmt(format_args!(" ({})", rc)).unwrap();
    }

    let zContext = shell_error_context(zSql, db);
    if !zContext.is_empty() {
        buffer.push_str(&zContext);
    }

    return buffer;
}

fn displayLinuxIoStats() {
    const aTrans: [C2RustUnnamed_18; 7] = [
        C2RustUnnamed_18 {
            zPattern: "rchar: ",
            zDesc: "Bytes received by read():",
        },
        C2RustUnnamed_18 {
            zPattern: "wchar: ",
            zDesc: "Bytes sent to write():",
        },
        C2RustUnnamed_18 {
            zPattern: "syscr: ",
            zDesc: "Read() system calls:",
        },
        C2RustUnnamed_18 {
            zPattern: "syscw: ",
            zDesc: "Write() system calls:",
        },
        C2RustUnnamed_18 {
            zPattern: "read_bytes: ",
            zDesc: "Bytes read from storage:",
        },
        C2RustUnnamed_18 {
            zPattern: "write_bytes: ",
            zDesc: "Bytes written to storage:",
        },
        C2RustUnnamed_18 {
            zPattern: "cancelled_write_bytes: ",
            zDesc: "Cancelled write bytes:",
        },
    ];

    let z = format!("/proc/{}/io", std::process::id());
    let mut in_0 = match File::open(z) {
        Err(_) => return,
        Ok(f) => f,
    };

    let mut z: [u8; 200] = [0; 200];
    loop {
        match in_0.read_exact(&mut z) {
            Err(_) => break,
            Ok(_) => {
                for trans in aTrans {
                    let n = trans.zPattern.len();
                    if z.starts_with(trans.zPattern.as_bytes()) {
                        print!("{:>36} {}", trans.zDesc, std::str::from_utf8(&z[n..]).unwrap(),);
                        break;
                    }
                }
            }
        }
    }
}

unsafe fn displayStatLine(zLabel: &str, zFormat: &'static str, iStatusCtrl: i32, bReset: i32) {
    let mut iCur: i64 = -1;
    let mut iHiwtr: i64 = -1;
    sqlite3_status64(iStatusCtrl, &mut iCur, &mut iHiwtr, bReset);

    println!("{:>36} {}", zLabel, strfmt!(zFormat, iCur, iHiwtr).unwrap());
}

unsafe fn display_stats(mut db: *mut sqlite3, pArg: *mut ShellState, bReset: i32) -> i32 {
    let mut iCur: i32 = 0;
    let mut iHiwtr: i32 = 0;
    if pArg.is_null() {
        return 0;
    }
    if !((*pArg).pStmt).is_null() && (*pArg).statsOn == 2 as u32 {
        let mut nCol: i32 = 0;
        let mut x: i32 = 0;
        let mut pStmt: *mut sqlite3_stmt = (*pArg).pStmt;
        let mut z: [i8; 100] = [0; 100];
        nCol = sqlite3_column_count(pStmt);
        println!("{:>36} {}", "Number of output columns:", nCol,);
        for i in 0..nCol {
            sqlite3_snprintf(
                ::std::mem::size_of::<[i8; 100]>() as u64 as i32,
                z.as_mut_ptr(),
                b"Column %d %nname:\0" as *const u8 as *const i8,
                i,
                &mut x as *mut i32,
            );
            println!(
                "{:>36} {}",
                cstr_to_string!(z.as_mut_ptr()),
                cstr_to_string!(sqlite3_column_name(pStmt, i))
            );
            sqlite3_snprintf(30, z.as_mut_ptr().offset(x as isize), b"declared type:\0" as *const u8 as *const i8);
            println!(
                "{:>36} {}",
                cstr_to_string!(z.as_mut_ptr()),
                cstr_to_string!(sqlite3_column_decltype(pStmt, i))
            );
        }
    }
    if (*pArg).statsOn == 3 {
        if !((*pArg).pStmt).is_null() {
            iCur = sqlite3_stmt_status((*pArg).pStmt, 4, bReset);
            println!("VM-steps: {}", iCur);
        }
        return 0;
    }
    displayStatLine("Memory Used:", "{iCur} (max {iHiwtr}) bytes{}", 0, bReset);
    displayStatLine("Number of Outstanding Allocations:", "{iCur} (max {iHiwtr})", 9, bReset);
    if (*pArg).shellFlgs & 0x1 != 0 {
        displayStatLine("Number of Pcache Pages Used:", "{iCur} (max {iHiwtr}) pages", 1, bReset);
    }
    displayStatLine("Number of Pcache Overflow Bytes:", "{iCur} (max {iHiwtr}) bytes", 2, bReset);
    displayStatLine("Largest Allocation:", "{iHiwtr} bytes", 5, bReset);
    displayStatLine("Largest Pcache Allocation:", "{iHiwtr} bytes", 7 as i32, bReset);
    if !db.is_null() {
        if (*pArg).shellFlgs & 0x2 as u32 != 0 {
            iCur = -1;
            iHiwtr = iCur;
            sqlite3_db_status(db, 0, &mut iCur, &mut iHiwtr, bReset);
            println!("Lookaside Slots Used:                {} (max {})", iCur, iHiwtr,);
            sqlite3_db_status(db, 4, &mut iCur, &mut iHiwtr, bReset);
            println!("Successful lookaside attempts:       {}", iHiwtr);
            sqlite3_db_status(db, 5, &mut iCur, &mut iHiwtr, bReset);
            println!("Lookaside failures due to size:      {}", iHiwtr);
            sqlite3_db_status(db, 6, &mut iCur, &mut iHiwtr, bReset);
            println!("Lookaside failures due to OOM:       {}", iHiwtr);
        }
        iCur = -1;
        iHiwtr = iCur;
        sqlite3_db_status(db, 1, &mut iCur, &mut iHiwtr, bReset);
        println!("Pager Heap Usage:                    {} bytes", iCur);
        iCur = -1;
        iHiwtr = iCur;
        sqlite3_db_status(db, 7 as i32, &mut iCur, &mut iHiwtr, 1);
        println!("Page cache hits:                     {}", iCur);
        iCur = -1;
        iHiwtr = iCur;
        sqlite3_db_status(db, 8, &mut iCur, &mut iHiwtr, 1);
        println!("Page cache misses:                   {}", iCur);
        iCur = -1;
        iHiwtr = iCur;
        sqlite3_db_status(db, 9, &mut iCur, &mut iHiwtr, 1);
        println!("Page cache writes:                   {}", iCur);
        iCur = -1;
        iHiwtr = iCur;
        sqlite3_db_status(db, 12, &mut iCur, &mut iHiwtr, 1);
        println!("Page cache spills:                   {}", iCur);
        iCur = -1;
        iHiwtr = iCur;
        sqlite3_db_status(db, 2, &mut iCur, &mut iHiwtr, bReset);
        println!("Schema Heap Usage:                   {} bytes", iCur);
        iCur = -1;
        iHiwtr = iCur;
        sqlite3_db_status(db, 3, &mut iCur, &mut iHiwtr, bReset);
        println!("Statement Heap/Lookaside Usage:      {} bytes", iCur);
    }
    if !((*pArg).pStmt).is_null() {
        let mut iHit: i32 = 0;
        let mut iMiss: i32 = 0;
        iCur = sqlite3_stmt_status((*pArg).pStmt, 1, bReset);
        println!("Fullscan Steps:                      {}", iCur);
        iCur = sqlite3_stmt_status((*pArg).pStmt, 2, bReset);
        println!("Sort Operations:                     {}", iCur);
        iCur = sqlite3_stmt_status((*pArg).pStmt, 3, bReset);
        println!("Autoindex Inserts:                   {}", iCur);
        iHit = sqlite3_stmt_status((*pArg).pStmt, 8, bReset);
        iMiss = sqlite3_stmt_status((*pArg).pStmt, 7 as i32, bReset);
        if iHit != 0 || iMiss != 0 {
            println!("Bloom filter bypass taken:           {}/{}", iHit, iHit + iMiss,);
        }
        iCur = sqlite3_stmt_status((*pArg).pStmt, 4, bReset);
        println!("Virtual Machine Steps:               {}", iCur);
        iCur = sqlite3_stmt_status((*pArg).pStmt, 5, bReset);
        println!("Reprepare operations:                {}", iCur);
        iCur = sqlite3_stmt_status((*pArg).pStmt, 6, bReset);
        println!("Number of times run:                 {}", iCur);
        iCur = sqlite3_stmt_status((*pArg).pStmt, 99, bReset);
        println!("Memory used by prepared stmt:        {}", iCur);
    }
    displayLinuxIoStats();
    return 0;
}
fn display_scanstats(mut _db: *mut sqlite3, mut _pArg: *mut ShellState) {}
fn str_in_array(zStr: &str, azArray: &[&'static str]) -> bool {
    for text in azArray {
        if text == &zStr {
            return true;
        }
    }
    false
}

unsafe fn explain_data_prepare(p: *mut ShellState, pSql: *mut sqlite3_stmt) {
    let mut zSql: *const i8 = 0 as *const i8;
    let mut z: *const i8 = 0 as *const i8;
    let mut abYield = Vec::new();
    let mut nAlloc: i32 = 0;
    let azNext = ["Next", "Prev", "VPrev", "VNext", "SorterNext"];
    let azYield = ["Yield", "SeekLT", "SeekGT", "RowSetRead", "Rewind"];
    let azGoto = ["Goto"];
    if sqlite3_column_count(pSql) != 8 {
        (*p).cMode = (*p).mode;
        return;
    }
    zSql = sqlite3_sql(pSql);
    if zSql.is_null() {
        return;
    }
    z = zSql;
    while *z as i32 == ' ' as i32
        || *z as i32 == '\t' as i32
        || *z as i32 == '\n' as i32
        || *z as i32 == '\u{c}' as i32
        || *z as i32 == '\r' as i32
    {
        z = z.offset(1);
    }
    if sqlite3_strnicmp(z, b"explain\0" as *const u8 as *const i8, 7 as i32) != 0 {
        (*p).cMode = (*p).mode;
        return;
    }
    let mut iOp = 0;
    while 100 == sqlite3_step(pSql) {
        let iAddr: i32 = sqlite3_column_int(pSql, 0);
        let zOp = cstr_to_string!(sqlite3_column_text(pSql, 1));
        let p2: i32 = sqlite3_column_int(pSql, 3);
        let p2op: i32 = p2 + (iOp - iAddr);
        if iOp >= nAlloc {
            if iOp == 0 {
                const explainCols: [*const i8; 8] = [
                    b"addr\0" as *const u8 as *const i8,
                    b"opcode\0" as *const u8 as *const i8,
                    b"p1\0" as *const u8 as *const i8,
                    b"p2\0" as *const u8 as *const i8,
                    b"p3\0" as *const u8 as *const i8,
                    b"p4\0" as *const u8 as *const i8,
                    b"p5\0" as *const u8 as *const i8,
                    b"comment\0" as *const u8 as *const i8,
                ];
                for jj in 0..8 {
                    if strcmp(sqlite3_column_name(pSql, jj), explainCols[jj as usize]) != 0 {
                        (*p).cMode = (*p).mode;
                        sqlite3_reset(pSql);
                        return;
                    }
                }
            }
            nAlloc += 100;
            (*p).aiIndent.resize(nAlloc as usize, 0);
            abYield.resize(nAlloc as usize, 0);
        }
        abYield[iOp as usize] = str_in_array(&zOp, &azYield) as i32;
        (*p).aiIndent[iOp as usize] = 0;
        (*p).nIndent = iOp + 1;
        if str_in_array(&zOp, &azNext) {
            for i in p2op..iOp {
                (*p).aiIndent[i as usize] += 2;
            }
        }
        if str_in_array(&zOp, &azGoto) && p2op < (*p).nIndent && (abYield[p2op as usize] != 0 || sqlite3_column_int(pSql, 2) != 0) {
            for i in p2op..iOp {
                (*p).aiIndent[i as usize] += 2;
            }
        }
        iOp += 1;
    }
    (*p).iIndent = 0;
    sqlite3_reset(pSql);
}

fn explain_data_delete(p: &mut ShellState) {
    p.aiIndent.clear();
    p.nIndent = 0;
    p.iIndent = 0;
}
static mut savedSelectTrace: u32 = 0;
static mut savedWhereTrace: u32 = 0;
unsafe fn disable_debug_trace_modes() {
    let mut zero: u32 = 0;
    sqlite3_test_control(31, 0, &mut savedSelectTrace as *mut u32);
    sqlite3_test_control(31, 1, &mut zero as *mut u32);
    sqlite3_test_control(31, 2, &mut savedWhereTrace as *mut u32);
    sqlite3_test_control(31, 3, &mut zero as *mut u32);
}
unsafe fn restore_debug_trace_modes() {
    sqlite3_test_control(31, 1, &mut savedSelectTrace as *mut u32);
    sqlite3_test_control(31, 3, &mut savedWhereTrace as *mut u32);
}
unsafe fn bind_table_init(p: *mut ShellState) {
    let mut wrSchema: i32 = 0;
    let mut defensiveMode: i32 = 0;
    sqlite3_db_config((*p).db, 1010, -1, &mut defensiveMode as *mut i32);
    sqlite3_db_config((*p).db, 1010, 0, 0);
    sqlite3_db_config((*p).db, 1011, -1, &mut wrSchema as *mut i32);
    sqlite3_db_config((*p).db, 1011, 1, 0);
    sqlite3_exec(
        (*p).db,
        b"CREATE TABLE IF NOT EXISTS temp.sqlite_parameters(\n  key TEXT PRIMARY KEY,\n  value\n) WITHOUT ROWID;\0" as *const u8
            as *const i8,
        None,
        std::ptr::null_mut(),
        0 as *mut *mut i8,
    );
    sqlite3_db_config((*p).db, 1011, wrSchema, 0);
    sqlite3_db_config((*p).db, 1010, defensiveMode, 0);
}
unsafe extern "C" fn bind_prepared_stmt(mut pArg: *mut ShellState, mut pStmt: *mut sqlite3_stmt) {
    let mut nVar: i32 = 0;
    let mut rc: i32 = 0;
    let mut pQ: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    nVar = sqlite3_bind_parameter_count(pStmt);
    if nVar == 0 {
        return;
    }
    if sqlite3_table_column_metadata(
        (*pArg).db,
        b"TEMP\0" as *const u8 as *const i8,
        b"sqlite_parameters\0" as *const u8 as *const i8,
        b"key\0" as *const u8 as *const i8,
        0 as *mut *const i8,
        0 as *mut *const i8,
        0 as *mut i32,
        0 as *mut i32,
        0 as *mut i32,
    ) != 0
    {
        return;
    }
    rc = sqlite3_prepare_v2(
        (*pArg).db,
        b"SELECT value FROM temp.sqlite_parameters WHERE key=?1\0" as *const u8 as *const i8,
        -1,
        &mut pQ,
        0 as *mut *const i8,
    );
    if rc != 0 || pQ.is_null() {
        return;
    }
    for i in 1..=nVar {
        let mut zNum: [i8; 30] = [0; 30];
        let mut zVar: *const i8 = sqlite3_bind_parameter_name(pStmt, i);
        if zVar.is_null() {
            sqlite3_snprintf(
                ::std::mem::size_of::<[i8; 30]>() as u64 as i32,
                zNum.as_mut_ptr(),
                b"?%d\0" as *const u8 as *const i8,
                i,
            );
            zVar = zNum.as_mut_ptr();
        }
        sqlite3_bind_text(pQ, 1, zVar, -1, None);
        if sqlite3_step(pQ) == 100 {
            sqlite3_bind_value(pStmt, i, sqlite3_column_value(pQ, 0));
        } else {
            sqlite3_bind_null(pStmt, i);
        }
        sqlite3_reset(pQ);
    }
    sqlite3_finalize(pQ);
}

fn print_box_line(mut N: i32) {
    let zDash = "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}";
    let nDash: i32 = zDash.len() as i32;
    while N > nDash {
        print!("{zDash}");
        N -= nDash;
    }
    print!("{}", &zDash[..N as usize]);
}

const BOX_24: char = '\u{2500}';
const BOX_13: char = '\u{2502}';
const BOX_23: char = '\u{250c}';
const BOX_34: char = '\u{2510}';
const BOX_12: char = '\u{2514}';
const BOX_14: char = '\u{2518}';
const BOX_123: char = '\u{251c}';
const BOX_134: char = '\u{2524}';
const BOX_234: char = '\u{252c}';
const BOX_124: char = '\u{2534}';
const BOX_1234: char = '\u{253c}';

fn print_box_row_separator(p: &ShellState, nArg: i32, zSep1: char, zSep2: char, zSep3: char) {
    let nArg = nArg as usize;
    if nArg > 0 {
        print!("{}", zSep1);
        print_box_line(p.actualWidth[0] + 2);
        for i in 1..nArg {
            print!("{}", zSep2);
            print_box_line(p.actualWidth[i] + 2);
        }
        print!("{}", zSep3);
    }
    println!();
}

/*
** z[] is a line of text that is to be displayed the .mode box or table or
** similar tabular formats.  z[] might contain control characters such
** as \n, \t, \f, or \r.
**
** Compute characters to display on the first line of z[].  Stop at the
** first \r, \n, or \f.  Expand \t into spaces.  Return a copy (obtained
** from malloc()) of that first line, which caller should free sometime.
** Write anything to display on the next line into *pzTail.  If this is
** the last line, write a NULL into *pzTail. (*pzTail is not allocated.)
*/
unsafe fn translateForDisplayAndDup(
    mut z: *const u8,           /* Input text to be transformed */
    mut pzTail: *mut *const u8, /* OUT: Tail of the input for next line */
    mxWidth: i32,               /* Max width.  0 means no limit */
    bWordWrap: u8,              /* If true, avoid breaking mid-word */
) -> *mut i8 {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    let mut n: i32 = 0;
    let mut zOut: *mut u8 = 0 as *mut u8;
    if z.is_null() {
        *pzTail = std::ptr::null();
        return std::ptr::null_mut();
    }
    let mxWidth = mxWidth.abs();
    let mxWidth = if mxWidth == 0 { 1000000 } else { mxWidth };

    n = 0;
    j = n;
    i = j;
    while n < mxWidth {
        if *z.offset(i as isize) >= b' ' {
            n += 1;
            loop {
                i += 1;
                j += 1;
                if !(*z.offset(i as isize) as i32 & 0xc0 == 0x80) {
                    break;
                }
            }
        } else {
            if !(*z.offset(i as isize) == b'\t') {
                break;
            }
            loop {
                n += 1;
                j += 1;
                if !(n & 7 as i32 != 0 && n < mxWidth) {
                    break;
                }
            }
            i += 1;
        }
    }
    if n >= mxWidth && bWordWrap != 0 {
        k = i;
        while k > i / 2 {
            if (*z.offset((k - 1) as isize)).is_ascii_whitespace() {
                break;
            }
            k -= 1;
        }
        if k <= i / 2 {
            k = i;
            while k > i / 2 {
                if (*z.offset((k - 1) as isize)).is_ascii_alphanumeric() != (*z.offset(k as isize)).is_ascii_alphabetic()
                    && *z.offset(k as isize) & 0xc0 != 0x80
                {
                    break;
                }
                k -= 1;
            }
        }
        if k <= i / 2 {
            k = i;
        } else {
            i = k;
            while *z.offset(i as isize) == b' ' {
                i += 1;
            }
        }
    } else {
        k = i;
    }
    if n >= mxWidth && *z.offset(i as isize) as i32 >= ' ' as i32 {
        *pzTail = &*z.offset(i as isize) as *const u8;
    } else if *z.offset(i as isize) as i32 == '\r' as i32 && *z.offset((i + 1) as isize) as i32 == '\n' as i32 {
        *pzTail = if *z.offset((i + 2) as isize) as i32 != 0 {
            &*z.offset((i + 2) as isize) as *const u8
        } else {
            std::ptr::null()
        };
    } else if *z.offset(i as isize) as i32 == 0 || *z.offset((i + 1) as isize) as i32 == 0 {
        *pzTail = std::ptr::null();
    } else {
        *pzTail = &*z.offset((i + 1) as isize) as *const u8;
    }
    zOut = malloc((j + 1) as u64) as *mut u8;
    shell_check_oom(zOut as *mut libc::c_void);
    n = 0;
    j = n;
    i = j;
    while i < k {
        if *z.offset(i as isize) as i32 >= ' ' as i32 {
            n += 1;
            loop {
                *zOut.offset(j as isize) = *z.offset(i as isize);
                i += 1;
                j += 1;
                if !(*z.offset(i as isize) as i32 & 0xc0 == 0x80) {
                    break;
                }
            }
        } else {
            if !(*z.offset(i as isize) as i32 == '\t' as i32) {
                break;
            }
            loop {
                n += 1;
                *zOut.offset(j as isize) = ' ' as i32 as u8;
                j += 1;
                if !(n & 7 as i32 != 0 && n < mxWidth) {
                    break;
                }
            }
            i += 1;
        }
    }
    *zOut.offset(j as isize) = 0;
    return zOut as *mut i8;
}

unsafe extern "C" fn quoted_column(mut pStmt: *mut sqlite3_stmt, mut i: i32) -> *mut i8 {
    match sqlite3_column_type(pStmt, i) {
        5 => return sqlite3_mprintf(b"NULL\0" as *const u8 as *const i8),
        1 | 2 => {
            return sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, sqlite3_column_text(pStmt, i));
        }
        3 => {
            return sqlite3_mprintf(b"%Q\0" as *const u8 as *const i8, sqlite3_column_text(pStmt, i));
        }
        4 => {
            let mut pStr: *mut sqlite3_str = sqlite3_str_new(0 as *mut sqlite3);
            let mut a: *const u8 = sqlite3_column_blob(pStmt, i) as *const u8;
            let mut n: i32 = sqlite3_column_bytes(pStmt, i);
            sqlite3_str_append(pStr, b"x'\0" as *const u8 as *const i8, 2);
            for j in 0..n {
                sqlite3_str_appendf(pStr, b"%02x\0" as *const u8 as *const i8, *a.offset(j as isize) as i32);
            }
            sqlite3_str_append(pStr, b"'\0" as *const u8 as *const i8, 1);
            return sqlite3_str_finish(pStr);
        }
        _ => {}
    }
    return std::ptr::null_mut();
}
unsafe extern "C" fn exec_prepared_stmt_columnar(mut p: *mut ShellState, mut pStmt: *mut sqlite3_stmt) {
    let mut current_block: u64;
    let mut nRow: i64 = 0;
    let mut nColumn: i32 = 0;
    let mut nAlloc: i64 = 0;
    let mut uz: *const u8 = std::ptr::null();
    let mut z: *const i8 = 0 as *const i8;
    let mut rc: i32 = 0;
    let mut nData: i64 = 0;
    let mut j: i32 = 0;
    let mut w: i32 = 0;
    let mut n: i32 = 0;
    let mut colSep = "";
    let mut rowSep = "";
    let mut bNextLine: i32 = 0;
    let mut bMultiLineRowExists: i32 = 0;
    let mut bw: i32 = (*p).cmOpts.bWordWrap as i32;
    let mut zEmpty: *const i8 = b"\0" as *const u8 as *const i8;
    let mut zShowNull: *const i8 = to_cstring!(((*p).nullValue).as_str());
    rc = sqlite3_step(pStmt);
    if rc != 100 {
        return;
    }
    nColumn = sqlite3_column_count(pStmt);
    nAlloc = (nColumn * 4) as i64;
    if nAlloc <= 0 {
        nAlloc = 1 as i64;
    }

    let mut azNextLine = Vec::new();

    let mut azData = Vec::with_capacity(nAlloc as usize);
    azNextLine.resize(nColumn as usize, 0 as *const u8);

    let mut azQuoted = Vec::new();
    if (*p).cmOpts.bQuote != 0 {
        azQuoted.resize(nColumn as usize, 0 as *mut i8);
    }

    let mut abRowDiv = Vec::with_capacity((nAlloc / nColumn as i64) as usize);

    if nColumn > (*p).nWidth {
        (*p).colWidth.resize(((nColumn + 1) * 2) as usize, 0);
        (*p).nWidth = nColumn;
        (*p).actualWidth = vec![0; nColumn as usize];
    }

    for i in 0..nColumn as usize {
        w = (*p).colWidth[i];
        if w < 0 {
            w = -w;
        }
        (*p).actualWidth[i] = w;
    }

    for i in 0..nColumn as usize {
        let mut zNotUsed: *const u8 = std::ptr::null();
        let mut wx: i32 = (*p).colWidth[i];
        if wx == 0 {
            wx = (*p).cmOpts.iWrap;
        }
        if wx < 0 {
            wx = -wx;
        }
        uz = sqlite3_column_name(pStmt, i as i32) as *const u8;
        azData.push(translateForDisplayAndDup(uz, &mut zNotUsed, wx, bw as u8));
    }

    loop {
        let mut useNextLine: i32 = bNextLine;
        bNextLine = 0;
        if (nRow + 2) * nColumn as i64 >= nAlloc {
            nAlloc *= 2 as i64;
            azData.resize(nAlloc as usize, 0 as *mut i8);
            abRowDiv.resize((nAlloc / nColumn as i64) as usize, 0);
        }

        abRowDiv[nRow as usize] = 1;
        nRow += 1;
        for i in 0..nColumn as usize {
            let mut wx_0: i32 = (*p).colWidth[i];
            if wx_0 == 0 {
                wx_0 = (*p).cmOpts.iWrap;
            }
            if wx_0 < 0 {
                wx_0 = -wx_0;
            }
            if useNextLine != 0 {
                uz = azNextLine[i];
                if uz.is_null() {
                    uz = zEmpty as *mut u8;
                }
            } else if (*p).cmOpts.bQuote != 0 {
                sqlite3_free(azQuoted[i] as _);
                azQuoted[i] = quoted_column(pStmt, i as i32);
                uz = azQuoted[i] as *const u8;
            } else {
                uz = sqlite3_column_text(pStmt, i as i32);
                if uz.is_null() {
                    uz = zShowNull as *mut u8;
                }
            }
            azData[(nRow as usize * nColumn as usize + i) as usize] = translateForDisplayAndDup(uz, &mut azNextLine[i], wx_0, bw as u8);
            if (azNextLine[i]).is_null() == false {
                bNextLine = 1;
                abRowDiv[(nRow - 1) as usize] = 0;
                bMultiLineRowExists = 1;
            }
        }

        if !(bNextLine != 0 || sqlite3_step(pStmt) == 100) {
            break;
        }
    }

    let nTotal = (nColumn as i64 * (nRow + 1)) as i32;
    for i in 0..nTotal as usize {
        z = azData[i];
        if z.is_null() {
            z = zEmpty as *mut i8;
        }
        n = strlenChar(z);
        j = (i as i64 % nColumn as i64) as i32;
        if n > (*p).actualWidth[j as usize] {
            (*p).actualWidth[j as usize] = n;
        }
    }

    if !(seenInterrupt != 0) {
        if !(nColumn == 0) {
            match (*p).cMode {
                1 => {
                    colSep = "  ";
                    rowSep = "\n";
                    if (*p).showHeader != 0 {
                        let nColumn = nColumn as usize;
                        for i in 0..nColumn {
                            w = (*p).actualWidth[i];
                            if (*p).colWidth[i] < 0 {
                                w = -w;
                            }
                            utf8_width_print(w, &cstr_to_string!(azData[i]));
                            if i == (nColumn - 1) {
                                println!();
                            } else {
                                print!("  ");
                            }
                        }

                        for i in 0..nColumn {
                            print_dashes((*p).actualWidth[i]);
                            if i == (nColumn - 1) {
                                println!();
                            } else {
                                print!("  ");
                            }
                        }
                    }
                }
                15 => {
                    colSep = " | ";
                    rowSep = " |\n";
                    print_row_separator(&*p, nColumn, '+');
                    print!("| ");
                    for i in 0..nColumn as usize {
                        let nColumn = nColumn as usize;
                        w = (*p).actualWidth[i];
                        n = strlenChar(azData[i]);
                        print!(
                            "{content1:width1$}{content2}{content3:width3$}",
                            width1 = ((w - n) / 2) as usize,
                            content1 = " ",
                            content2 = cstr_to_string!(azData[i]),
                            width3 = ((w - n + 1) / 2) as usize,
                            content3 = " ",
                        );
                        if i == (nColumn - 1) {
                            println!(" |");
                        } else {
                            print!(" | ");
                        }
                    }

                    print_row_separator(&*p, nColumn, '+');
                }
                14 => {
                    colSep = " | ";
                    rowSep = " |\n";
                    print!("| ");

                    for i in 0..nColumn as usize {
                        let nColumn = nColumn as usize;
                        w = (*p).actualWidth[i];
                        n = strlenChar(azData[i]);
                        println!(
                            "{content1:width1$}{content2}{content3:width3$}",
                            width1 = ((w - n) / 2) as usize,
                            content1 = " ",
                            content2 = cstr_to_string!(azData[i]),
                            width3 = ((w - n + 1) / 2) as usize,
                            content3 = " ",
                        );
                        if i == (nColumn - 1) {
                            println!(" |");
                        } else {
                            print!(" | ");
                        }
                    }
                    print_row_separator(&*p, nColumn, '|');
                }
                16 => {
                    colSep = concatcp!(" ", BOX_13, " ");
                    rowSep = concatcp!(" ", BOX_13, "\n");
                    print_box_row_separator(&*p, nColumn, BOX_23, BOX_234, BOX_34);
                    print!("{} ", BOX_13);
                    for i in 0..nColumn as usize {
                        let nColumn = nColumn as usize;
                        w = (*p).actualWidth[i];
                        n = strlenChar(azData[i]);

                        print!(
                            "{content1:width1$}{content2}{content3:width3$}{content4}",
                            width1 = ((w - n) / 2) as usize,
                            content1 = " ",
                            content2 = cstr_to_string!(azData[i]),
                            width3 = ((w - n + 1) / 2) as usize,
                            content3 = " ",
                            content4 = if i == (nColumn - 1) {
                                concatcp!(" ", BOX_13, "\n")
                            } else {
                                concatcp!(" ", BOX_13, " ")
                            },
                        );
                    }
                    print_box_row_separator(&*p, nColumn, BOX_123, BOX_1234, BOX_134);
                }
                _ => {}
            }

            let mut i = nColumn as usize;
            let mut j: isize = 0;
            let nColumn = nColumn as usize;
            loop {
                if !(i < nTotal as usize) {
                    current_block = 16107425721173356396;
                    break;
                }
                if j == 0 && (*p).cMode != 1 {
                    print!("{}", if (*p).cMode == 16 { concatcp!(BOX_13, " ") } else { "| " },);
                }
                z = azData[i];
                if z.is_null() {
                    z = to_cstring!((*p).nullValue.as_str());
                }
                w = (*p).actualWidth[j as usize];
                if (*p).colWidth[j as usize] < 0 {
                    w = -w;
                }
                utf8_width_print(w, &cstr_to_string!(z));
                if j as usize == nColumn - 1 {
                    print!("{}", rowSep);
                    if bMultiLineRowExists != 0 && abRowDiv[(i / nColumn - 1)] != 0 && (i + 1) < nTotal as usize {
                        if (*p).cMode == 15 {
                            print_row_separator(&*p, nColumn as i32, '+');
                        } else if (*p).cMode == 16 {
                            print_box_row_separator(&*p, nColumn as i32, BOX_123, BOX_1234, BOX_134);
                        } else if (*p).cMode == 1 {
                            println!();
                        }
                    }
                    j = -1;
                    if seenInterrupt != 0 {
                        current_block = 3475906965431124488;
                        break;
                    }
                } else {
                    print!("{}", colSep);
                }
                i += 1;
                j += 1;
            }
            match current_block {
                3475906965431124488 => {}
                _ => {
                    if (*p).cMode == 15 {
                        print_row_separator(&*p, nColumn as i32, '+');
                    } else if (*p).cMode == 16 {
                        print_box_row_separator(&*p, nColumn as i32, BOX_12, BOX_124, BOX_14);
                    }
                }
            }
        }
    }
    if seenInterrupt != 0 {
        println!("Interrupt");
    }
    nData = (nRow + 1) * nColumn as i64;
    for i in 0..nData as usize {
        z = azData[i];
        if z != zEmpty && z != zShowNull {
            free(azData[i] as _);
        }
    }

    if !azQuoted.is_empty() {
        for i in 0..nColumn as usize {
            sqlite3_free(azQuoted[i] as *mut libc::c_void);
        }
    }
}

unsafe fn exec_prepared_stmt(pArg: *mut ShellState, pStmt: *mut sqlite3_stmt) {
    if (*pArg).cMode == 1 || (*pArg).cMode == 15 || (*pArg).cMode == 16 || (*pArg).cMode == 14 {
        exec_prepared_stmt_columnar(pArg, pStmt);
        return;
    }
    let mut rc = sqlite3_step(pStmt);
    if 100 == rc {
        let mut nCol = sqlite3_column_count(pStmt) as usize;

        let mut azCols = Vec::new();
        azCols.resize(nCol, 0 as *mut i8);

        let mut azVals = Vec::new();
        azVals.resize(nCol, 0 as *mut i8);

        let mut aiTypes = Vec::new();
        aiTypes.resize(nCol, 0);

        assert!(::std::mem::size_of::<i32>() as u64 <= ::std::mem::size_of::<*mut i8>() as u64);

        for i in 0..nCol {
            azCols[i] = sqlite3_column_name(pStmt, i as i32) as *mut i8;
        }

        let mut nRow: u64 = 0;
        loop {
            nRow += 1;
            for i in 0..nCol {
                let x = sqlite3_column_type(pStmt, i as i32);
                aiTypes[i] = x;
                if x == 4 && !pArg.is_null() && ((*pArg).cMode == 5 || (*pArg).cMode == 6) {
                    azVals[i] = b"\0" as *const u8 as *const i8 as *mut i8;
                } else {
                    azVals[i] = sqlite3_column_text(pStmt, i as i32) as *mut i8;
                }
                if azVals[i].is_null() && aiTypes[i] != 5 {
                    rc = 7;
                    break;
                }
            }
            if 100 == rc {
                if shell_callback(
                    pArg as *mut libc::c_void,
                    nCol as i32,
                    azVals.as_mut_ptr(),
                    azCols.as_mut_ptr(),
                    aiTypes.as_mut_ptr(),
                ) != 0
                {
                    rc = 4;
                } else {
                    rc = sqlite3_step(pStmt);
                }
            }
            if !(100 == rc) {
                break;
            }
        }
        if (*pArg).cMode == 13 {
            println!("]");
        } else if (*pArg).cMode == 17 {
            print!("{} row{}", nRow, if nRow != 1 { "s" } else { "" });
        }
    }
}

unsafe fn expertHandleSQL(mut pState: *mut ShellState, zSql: &str, rpzErr: &mut String) -> i32 {
    assert!(!((*pState).expert.pExpert).is_null());
    // assert!(pzErr.is_null() || (*pzErr).is_null());
    let mut pzErr = std::ptr::null_mut();
    let rc = sqlite3_expert_sql((*pState).expert.pExpert, to_cstring!(zSql), pzErr);

    *rpzErr = cstr_to_string!(pzErr).to_string();
    return rc;
}
unsafe fn expertFinish(pState: *mut ShellState, bCancel: i32, rpzErr: &mut String) -> i32 {
    let mut rc: i32 = 0;
    let mut p: *mut sqlite3expert = (*pState).expert.pExpert;
    assert!(!p.is_null());
    assert!(bCancel != 0 || rpzErr.is_empty());
    if bCancel == 0 {
        let bVerbose: i32 = (*pState).expert.bVerbose;
        let mut pzErr = std::ptr::null_mut();

        rc = sqlite3_expert_analyze(p, pzErr);
        *rpzErr = cstr_to_string!(pzErr).to_string();

        if rc == 0 {
            let nQuery: i32 = sqlite3_expert_count(p);
            if bVerbose != 0 {
                let zCand: *const i8 = sqlite3_expert_report(p, 0, 4);
                println!("-- Candidates -----------------------------");
                println!("{}", cstr_to_string!(zCand));
            }
            for i in 0..nQuery {
                let zSql: *const i8 = sqlite3_expert_report(p, i, 1);
                let mut zIdx: *const i8 = sqlite3_expert_report(p, i, 2);
                let zEQP: *const i8 = sqlite3_expert_report(p, i, 3);
                if zIdx.is_null() {
                    zIdx = b"(no new indexes)\n\0" as *const u8 as *const i8;
                }
                if bVerbose != 0 {
                    println!("-- Query {} --------------------------------", i + 1,);
                    println!("{}\n", cstr_to_string!(zSql));
                }
                println!("{}", cstr_to_string!(zIdx));
                println!("{}", cstr_to_string!(zEQP));
            }
        }
    }
    sqlite3_expert_destroy(p);
    (*pState).expert.pExpert = 0 as *mut sqlite3expert;
    return rc;
}

unsafe fn expertDotCommand(pState: *mut ShellState, azArg: &Vec<String>) -> i32 {
    let mut zErr: *mut i8 = std::ptr::null_mut();
    let mut iSample: i32 = 0;
    assert!(((*pState).expert.pExpert).is_null());
    memset(
        &mut (*pState).expert as *mut ExpertInfo as *mut libc::c_void,
        0,
        ::std::mem::size_of::<ExpertInfo>() as u64,
    );
    let mut rc: i32 = 0;
    let mut i: usize = 1;
    while rc == 0 && i < azArg.len() {
        // skip -
        let z = {
            let z = azArg[i as usize].as_str();
            if z.starts_with("--") {
                &z[1..]
            } else {
                z
            }
        };

        if z == "-verbose" {
            (*pState).expert.bVerbose = 1;
        } else if z == "-sample" {
            if i == azArg.len() - 1 {
                eprintln!("option requires an argument: {}", z);
                rc = 1;
            } else {
                i += 1;
                iSample = integerValue(&azArg[i]) as i32;
                if iSample < 0 || iSample > 100 {
                    eprintln!("value out of range: {}", azArg[i]);
                    rc = 1;
                }
            }
        } else {
            eprintln!("unknown option: {}", z);
            rc = 1;
        }
    }

    if rc == 0 {
        (*pState).expert.pExpert = sqlite3_expert_new((*pState).db, &mut zErr);
        if ((*pState).expert.pExpert).is_null() {
            if !zErr.is_null() {
                eprintln!("sqlite3_expert_new: {}", cstr_to_string!(zErr as *const i8))
            } else {
                eprintln!("sqlite3_expert_new: out of memory")
            }
            rc = 1;
        } else {
            sqlite3_expert_config((*pState).expert.pExpert, 1, iSample);
        }
    }
    sqlite3_free(zErr as *mut libc::c_void);
    return rc;
}

/*
** Execute a statement or set of statements.  Print
** any result rows/columns depending on the current mode
** set via the supplied callback.
**
** This is very similar to SQLite's built-in sqlite3_exec()
** function except it takes a slightly different callback
** and callback data argument.
*/
unsafe fn shell_exec(mut pArg: *mut ShellState, zSql: &str, pzErrMsg: &mut String) -> i32 {
    if !((*pArg).expert.pExpert).is_null() {
        let rc = expertHandleSQL(pArg, zSql, pzErrMsg);
        return expertFinish(pArg, (rc != 0) as i32, pzErrMsg);
    }

    let mut result = zSql.to_string();
    result.push('\0');
    let mut zSql = result.as_bytes().as_bstr();

    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut db: *mut sqlite3 = (*pArg).db;
    let mut rc: i32 = 0;
    let mut zStmtSql = B(b"");
    while zSql[0] != 0 && rc == 0 {
        let mut temp2: *const i8 = std::ptr::null();
        rc = sqlite3_prepare_v2(db, zSql.as_ptr() as _, (zSql.len() - 1) as i32, &mut pStmt, &mut temp2);
        let mut zLeftover = if temp2.is_null() {
            let nul = &zSql[(zSql.len() - 1)..];
            CStr::from_ptr(nul.as_ptr() as _)
        } else {
            CStr::from_ptr(temp2)
        };

        if 0 != rc {
            if !pzErrMsg.is_empty() {
                *pzErrMsg = save_err_msg(db, "in prepare", rc, zSql[..zSql.len() - 1].to_str_unchecked());
            }
        } else if pStmt.is_null() {
            zSql = zLeftover.to_bytes_with_nul().trim_start().as_bstr();
            continue;
        } else {
            zStmtSql = {
                let mut result = sqlite3_sql(pStmt);
                if result.is_null() {
                    b""
                } else {
                    let cstr = CStr::from_ptr(result);
                    cstr.to_bytes()
                }
            };
            zStmtSql = zStmtSql.trim_start();
            if !pArg.is_null() {
                (*pArg).pStmt = pStmt;
                (*pArg).cnt = 0;
            }
            if !pArg.is_null() && (*pArg).shellFlgs & 0x40 != 0 {
                if zStmtSql.is_empty() == false {
                    println!("{}", zStmtSql.as_bstr());
                } else {
                    println!("{}", zSql);
                }
            }

            if !pArg.is_null() && (*pArg).autoEQP as i32 != 0 && sqlite3_stmt_isexplain(pStmt) == 0 {
                let mut pExplain: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                let mut triggerEQP: i32 = 0;
                disable_debug_trace_modes();
                sqlite3_db_config(db, 1008, -1, &mut triggerEQP as *mut i32);
                if (*pArg).autoEQP >= 2 {
                    sqlite3_db_config(db, 1008, 1, 0);
                }
                let zEQP = format!("EXPLAIN QUERY PLAN {}\0", zStmtSql.as_bstr());
                rc = sqlite3_prepare_v2(db, zEQP.as_ptr() as _, -1, &mut pExplain, 0 as *mut *const i8);
                if rc == 0 {
                    while sqlite3_step(pExplain) == 100 {
                        let mut zEQPLine: *const i8 = sqlite3_column_text(pExplain, 3) as *const i8;
                        let mut iEqpId: i32 = sqlite3_column_int(pExplain, 0);
                        let mut iParentId: i32 = sqlite3_column_int(pExplain, 1);
                        let zEQPLine = if zEQPLine.is_null() {
                            Cow::from("")
                        } else {
                            cstr_to_string!(zEQPLine)
                        };
                        if zEQPLine.starts_with('-') {
                            eqp_render(&mut (*pArg).sGraph);
                        }
                        eqp_append(&mut *pArg, iEqpId, iParentId, &zEQPLine);
                    }
                    eqp_render(&mut (*pArg).sGraph);
                }
                sqlite3_finalize(pExplain);
                drop(zEQP);

                if (*pArg).autoEQP as i32 >= 3 {
                    let zEQP = format!("EXPLAIN {}\0", zStmtSql.as_bstr());
                    rc = sqlite3_prepare_v2(db, zEQP.as_ptr() as _, -1, &mut pExplain, 0 as *mut *const i8);
                    if rc == 0 {
                        (*pArg).cMode = 9;
                        explain_data_prepare(pArg, pExplain);
                        exec_prepared_stmt(pArg, pExplain);
                        explain_data_delete(&mut *pArg);
                    }
                    sqlite3_finalize(pExplain);
                }
                if (*pArg).autoEQP >= 2 && triggerEQP == 0 {
                    sqlite3_db_config(db, 1008, 0, 0);
                    sqlite3_finalize(pStmt);
                    sqlite3_prepare_v2(db, zSql.as_ptr() as _, -1, &mut pStmt, 0 as *mut *const i8);
                    if !pArg.is_null() {
                        (*pArg).pStmt = pStmt;
                    }
                }
                restore_debug_trace_modes();
            }
            if !pArg.is_null() {
                (*pArg).cMode = (*pArg).mode;
                if (*pArg).autoExplain != 0 {
                    if sqlite3_stmt_isexplain(pStmt) == 1 {
                        (*pArg).cMode = 9;
                    }
                    if sqlite3_stmt_isexplain(pStmt) == 2 {
                        (*pArg).cMode = 12;
                    }
                }
                if (*pArg).cMode == 9 {
                    explain_data_prepare(pArg, pStmt);
                }
            }
            bind_prepared_stmt(pArg, pStmt);
            exec_prepared_stmt(pArg, pStmt);
            explain_data_delete(&mut *pArg);
            eqp_render(&mut (*pArg).sGraph);
            if !pArg.is_null() && (*pArg).statsOn != 0 {
                display_stats(db, pArg, 0);
            }
            if !pArg.is_null() && (*pArg).scanstatsOn as i32 != 0 {
                display_scanstats(db, pArg);
            }
            let rc2 = sqlite3_finalize(pStmt);
            if rc != 7 {
                rc = rc2;
            }
            if rc == 0 {
                zSql = zLeftover.to_bytes_with_nul().trim_start().as_bstr();
            } else if !pzErrMsg.is_empty() {
                *pzErrMsg = save_err_msg(db, "stepping", rc, "");
            }
            if !pArg.is_null() {
                (*pArg).pStmt = 0 as *mut sqlite3_stmt;
            }
        }
    }
    return rc;
}
unsafe extern "C" fn freeColumnList(mut azCol: *mut *mut i8) {
    let mut i: i32 = 0;
    i = 1;
    while !(*azCol.offset(i as isize)).is_null() {
        sqlite3_free(*azCol.offset(i as isize) as *mut libc::c_void);
        i += 1;
    }
    sqlite3_free(azCol as *mut libc::c_void);
}

/*
** Return a list of pointers to strings which are the names of all
** columns in table zTab.   The memory to hold the names is dynamically
** allocated and must be released by the caller using a subsequent call
** to freeColumnList().
**
** The azCol[0] entry is usually NULL.  However, if zTab contains a rowid
** value that needs to be preserved, then azCol[0] is filled in with the
** name of the rowid column.
**
** The first regular column in the table is azCol[1].  The list is terminated
** by an entry with azCol[i]==0.
*/
unsafe fn tableColumnList(mut p: *mut ShellState, zTab: &str) -> Vec<String> {
    let mut azCol = Vec::new();
    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;

    let mut nCol: i32 = 0;
    let mut nPK: i32 = 0;
    let mut isIPK = false;
    let mut preserveRowid: i32 = ((*p).shellFlgs & 0x8 != 0) as i32;

    let zSql = format!("PRAGMA table_info={}\0", zTab); // FIXME: use %Q as quoted text
    let mut rc = sqlite3_prepare_v2((*p).db, zSql.as_ptr() as _, -1, &mut pStmt, 0 as *mut *const i8);
    drop(zSql);

    if rc != 0 {
        return azCol;
    }

    azCol.push(String::new());

    while sqlite3_step(pStmt) == 100 {
        nCol += 1;

        let colName = cstr_to_string!(sqlite3_column_text(pStmt, 1));
        azCol.push(colName.to_string());

        if sqlite3_column_int(pStmt, 5) != 0 {
            nPK += 1;
            if nPK == 1 && sqlite3_stricmp(sqlite3_column_text(pStmt, 2) as *const i8, b"INTEGER\0" as *const u8 as *const i8) == 0 {
                isIPK = true;
            } else {
                isIPK = false;
            }
        }
    }
    sqlite3_finalize(pStmt);

    if azCol.len() == 1 {
        azCol.clear();
        return azCol;
    }

    if preserveRowid != 0 && isIPK {
        let zSql = sqlite3_mprintf(
            b"SELECT 1 FROM pragma_index_list(%Q) WHERE origin='pk'\0" as *const u8 as *const i8,
            zTab,
        );
        shell_check_oom(zSql as *mut libc::c_void);
        rc = sqlite3_prepare_v2((*p).db, zSql, -1, &mut pStmt, 0 as *mut *const i8);
        sqlite3_free(zSql as *mut libc::c_void);
        if rc != 0 {
            azCol.clear();
            return azCol;
        }
        rc = sqlite3_step(pStmt);
        sqlite3_finalize(pStmt);
        preserveRowid = (rc == 100) as i32;
    }
    if preserveRowid != 0 {
        const azRowid: [&'static str; 3] = ["rowid", "_rowid_", "oid"];
        for j in 0..3 {
            let mut n = 0;
            for i in 1..=nCol {
                n = i;
                if azRowid[j as usize].eq_ignore_ascii_case(&azCol[i as usize]) {
                    break;
                }
            }
            if n > nCol {
                rc = sqlite3_table_column_metadata(
                    (*p).db,
                    0 as *const i8,
                    to_cstring!(zTab),
                    to_cstring!(azRowid[j as usize]),
                    0 as *mut *const i8,
                    0 as *mut *const i8,
                    0 as *mut i32,
                    0 as *mut i32,
                    0 as *mut i32,
                );
                if rc == 0 {
                    azCol[0] = azRowid[j as usize].to_string();
                }
                break;
            }
        }
    }
    return azCol;
}
unsafe extern "C" fn toggleSelectOrder(mut db: *mut sqlite3) {
    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut iSetting: i32 = 0;
    let mut zStmt: [i8; 100] = [0; 100];
    sqlite3_prepare_v2(
        db,
        b"PRAGMA reverse_unordered_selects\0" as *const u8 as *const i8,
        -1,
        &mut pStmt,
        0 as *mut *const i8,
    );
    if sqlite3_step(pStmt) == 100 {
        iSetting = sqlite3_column_int(pStmt, 0);
    }
    sqlite3_finalize(pStmt);
    sqlite3_snprintf(
        ::std::mem::size_of::<[i8; 100]>() as u64 as i32,
        zStmt.as_mut_ptr(),
        b"PRAGMA reverse_unordered_selects(%d)\0" as *const u8 as *const i8,
        (iSetting == 0) as i32,
    );
    sqlite3_exec(db, zStmt.as_mut_ptr(), None, std::ptr::null_mut(), 0 as *mut *mut i8);
}
unsafe extern "C" fn dump_callback(mut pArg: *mut libc::c_void, mut nArg: i32, mut azArg: *mut *mut i8, _azNotUsed: *mut *mut i8) -> i32 {
    let mut rc: i32 = 0;

    let mut p: *mut ShellState = pArg as *mut ShellState;
    let mut dataOnly: i32 = 0;
    let mut noSys: i32 = 0;
    if nArg != 3 || azArg.is_null() {
        return 0;
    }

    let zTable = cstr_to_string!(*azArg.offset(0));
    let zType = cstr_to_string!(*azArg.offset(1));
    let zSql = cstr_to_string!(*azArg.offset(2 as isize));

    dataOnly = ((*p).shellFlgs & 0x100 != 0) as i32;
    noSys = ((*p).shellFlgs & 0x200 != 0) as i32;
    if zTable == "sqlite_sequence" && noSys == 0 {
        if dataOnly == 0 {
            println!("DELETE FROM sqlite_sequence;");
        }
    } else if sqrite::strglob("sqlite_stat?", &zTable) && noSys == 0 {
        if dataOnly == 0 {
            println!("ANALYZE sqlite_schema;");
        }
    } else if zTable.starts_with("sqlite_") {
        return 0;
    } else {
        if !(dataOnly != 0) {
            if zSql.starts_with("CREATE VIRTUAL TABLE") {
                if (*p).writableSchema == 0 {
                    println!("PRAGMA writable_schema=ON;");
                    (*p).writableSchema = 1;
                }

                // FIXME: zTable & zSQL must be escape character quote (')
                let zIns = format!(
                    "INSERT INTO sqlite_schema(type,name,tbl_name,rootpage,sql)VALUES('table','{}','{}',0,'{}');",
                    &zTable, &zTable, zSql,
                );

                println!("{}", zIns);

                return 0;
            } else {
                printSchemaLine(&zSql, ";\n");
            }
        }
    }

    if zType == "table" {
        let mut savedDestTable = String::new();
        let mut savedMode: i32 = 0;
        let razCol = tableColumnList(p, &zTable);
        if razCol.is_empty() {
            (*p).nErr += 1;
            return 0;
        }

        let mut sTable: ShellText = ShellText::new();
        sTable.append(&zTable, quoteChar(&zTable));
        if razCol[0].is_empty() == false {
            sTable.append("(", 0);
            sTable.append(&razCol[0], 0);
            for i in 1..razCol.len() {
                sTable.append(",", 0);
                sTable.append(&razCol[i as usize], quoteChar(&razCol[i as usize]));
            }
            sTable.append(")", 0);
        }
        let mut sSelect = ShellText::new();
        sSelect.append("SELECT ", 0);
        if razCol[0].is_empty() == false {
            sSelect.append(&razCol[0], 0);
            sSelect.append(",", 0);
        }
        for i in 1..razCol.len() {
            sSelect.append(&razCol[i as usize], quoteChar(&razCol[i as usize]));
            if razCol[(i + 1) as usize].is_empty() == false {
                sSelect.append(",", 0);
            }
        }
        drop(razCol);

        sSelect.append(" FROM ", 0);
        sSelect.append(&zTable, quoteChar(&zTable));
        savedDestTable = (*p).zDestTable.clone();
        savedMode = (*p).mode;
        (*p).zDestTable = sTable.to_string();
        (*p).cMode = 5;
        (*p).mode = 5;
        let mut pzErrMsg = String::new();
        rc = shell_exec(p, sSelect.as_str(), &mut pzErrMsg);
        if rc & 0xff == 11 {
            println!("/****** CORRUPTION ERROR *******/");
            toggleSelectOrder((*p).db);
            shell_exec(p, sSelect.as_str(), &mut pzErrMsg);
            toggleSelectOrder((*p).db);
        }
        (*p).zDestTable = savedDestTable.clone();
        (*p).mode = savedMode;

        sSelect.clear();
        if rc != 0 {
            (*p).nErr += 1;
        }
    }
    return 0;
}

unsafe fn run_schema_dump_query(p: *mut ShellState, zQuery: &str) -> i32 {
    let mut rc: i32 = 0;
    let mut zErr: *mut i8 = std::ptr::null_mut();

    let czQuery = to_cstring!(zQuery);
    rc = sqlite3_exec((*p).db, czQuery, Some(dump_callback), p as *mut libc::c_void, &mut zErr);
    if rc == 11 {
        println!("/****** CORRUPTION ERROR *******/");
        if !zErr.is_null() {
            println!("/****** {} ******/", cstr_to_string!(zErr));
            sqlite3_free(zErr as *mut libc::c_void);
            zErr = std::ptr::null_mut();
        }
        let zQ2 = format!("{} ORDER BY rowid DESC\0", zQuery);
        rc = sqlite3_exec(
            (*p).db,
            zQ2.as_ptr() as *const i8,
            Some(dump_callback),
            p as *mut libc::c_void,
            &mut zErr,
        );
        if rc != 0 {
            println!("/****** ERROR: {} ******/", cstr_to_string!(zErr));
        } else {
            rc = 11;
        }
        sqlite3_free(zErr as *mut libc::c_void);
    }
    return rc;
}
const azHelp: [&str; 176] = [
    ".auth ON|OFF             Show authorizer callbacks",
    ".backup ?DB? FILE        Backup DB (default \"main\") to FILE",
    "   Options:",
    "       --append            Use the appendvfs",
    "       --async             Write to FILE without journal and fsync()",
    ".bail on|off             Stop after hitting an error.  Default OFF",
    ".binary on|off           Turn binary output on or off.  Default OFF",
    ".cd DIRECTORY            Change the working directory to DIRECTORY",
    ".changes on|off          Show number of rows changed by SQL",
    ".check GLOB              Fail if output since .testcase does not match",
    ".clone NEWDB             Clone data into NEWDB from the existing database",
    ".connection [close] [#]  Open or close an auxiliary database connection",
    ".databases               List names and files of attached databases",
    ".dbconfig ?op? ?val?     List or change sqlite3_db_config() options",
    ".dbinfo ?DB?             Show status information about the database",
    ".dump ?OBJECTS?          Render database content as SQL",
    "   Options:",
    "     --data-only            Output only INSERT statements",
    "     --newlines             Allow unescaped newline characters in output",
    "     --nosys                Omit system tables (ex: \"sqlite_stat1\")",
    "     --preserve-rowids      Include ROWID values in the output",
    "   OBJECTS is a LIKE pattern for tables, indexes, triggers or views to dump",
    "   Additional LIKE patterns can be given in subsequent arguments",
    ".echo on|off             Turn command echo on or off",
    ".eqp on|off|full|...     Enable or disable automatic EXPLAIN QUERY PLAN",
    "   Other Modes:",
    "      trigger               Like \"full\" but also show trigger bytecode",
    ".excel                   Display the output of next command in spreadsheet",
    "   --bom                   Put a UTF8 byte-order mark on intermediate file",
    ".exit ?CODE?             Exit this program with return-code CODE",
    ".expert                  EXPERIMENTAL. Suggest indexes for queries",
    ".explain ?on|off|auto?   Change the EXPLAIN formatting mode.  Default: auto",
    ".filectrl CMD ...        Run various sqlite3_file_control() operations",
    "   --schema SCHEMA         Use SCHEMA instead of \"main\"",
    "   --help                  Show CMD details",
    ".fullschema ?--indent?   Show schema and the content of sqlite_stat tables",
    ".headers on|off          Turn display of headers on or off",
    ".help ?-all? ?PATTERN?   Show help text for PATTERN",
    ".import FILE TABLE       Import data from FILE into TABLE",
    "   Options:",
    "     --ascii               Use \\037 and \\036 as column and row separators",
    "     --csv                 Use , and \\n as column and row separators",
    "     --skip N              Skip the first N rows of input",
    "     --schema S            Target table to be S.TABLE",
    "     -v                    \"Verbose\" - increase auxiliary output",
    "   Notes:",
    "     *  If TABLE does not exist, it is created.  The first row of input",
    "        determines the column names.",
    "     *  If neither --csv or --ascii are used, the input mode is derived",
    "        from the \".mode\" output mode",
    "     *  If FILE begins with \"|\" then it is a command that generates the",
    "        input text.",
    ".imposter INDEX TABLE    Create imposter table TABLE on index INDEX",
    ".indexes ?TABLE?         Show names of indexes",
    "                           If TABLE is specified, only show indexes for",
    "                           tables matching TABLE using the LIKE operator.",
    ".limit ?LIMIT? ?VAL?     Display or change the value of an SQLITE_LIMIT",
    ".lint OPTIONS            Report potential schema issues.",
    "     Options:",
    "        fkey-indexes     Find missing foreign key indexes",
    ".load FILE ?ENTRY?       Load an extension library",
    ".log FILE|off            Turn logging on or off.  FILE can be stderr/stdout",
    ".mode MODE ?OPTIONS?     Set output mode",
    "   MODE is one of:",
    "     ascii       Columns/rows delimited by 0x1F and 0x1E",
    "     box         Tables using unicode box-drawing characters",
    "     csv         Comma-separated values",
    "     column      Output in columns.  (See .width)",
    "     html        HTML <table> code",
    "     insert      SQL insert statements for TABLE",
    "     json        Results in a JSON array",
    "     line        One value per line",
    "     list        Values delimited by \"|\"",
    "     markdown    Markdown table format",
    "     qbox        Shorthand for \"box --width 60 --quote\"",
    "     quote       Escape answers as for SQL",
    "     table       ASCII-art table",
    "     tabs        Tab-separated values",
    "     tcl         TCL list elements",
    "   OPTIONS: (for columnar modes or insert mode):",
    "     --wrap N       Wrap output lines to no longer than N characters",
    "     --wordwrap B   Wrap or not at word boundaries per B (on/off)",
    "     --ww           Shorthand for \"--wordwrap 1\"",
    "     --quote        Quote output text as SQL literals",
    "     --noquote      Do not quote output text",
    "     TABLE          The name of SQL table used for \"insert\" mode",
    ".nonce STRING            Suspend safe mode for one command if nonce matches",
    ".nullvalue STRING        Use STRING in place of NULL values",
    ".once ?OPTIONS? ?FILE?   Output for the next SQL command only to FILE",
    "     If FILE begins with '|' then open as a pipe",
    "       --bom  Put a UTF8 byte-order mark at the beginning",
    "       -e     Send output to the system text editor",
    "       -x     Send output as CSV to a spreadsheet (same as \".excel\")",
    ".open ?OPTIONS? ?FILE?   Close existing database and reopen FILE",
    "     Options:",
    "        --append        Use appendvfs to append database to the end of FILE",
    "        --deserialize   Load into memory using sqlite3_deserialize()",
    "        --hexdb         Load the output of \"dbtotxt\" as an in-memory d",
    "        --maxsize N     Maximum size for --hexdb or --deserialized database",
    "        --new           Initialize FILE to an empty database",
    "        --nofollow      Do not follow symbolic links",
    "        --readonly      Open FILE readonly",
    "        --zip           FILE is a ZIP archive",
    ".output ?FILE?           Send output to FILE or stdout if FILE is omitted",
    "   If FILE begins with '|' then open it as a pipe.",
    "   Options:",
    "     --bom                 Prefix output with a UTF8 byte-order mark",
    "     -e                    Send output to the system text editor",
    "     -x                    Send output as CSV to a spreadsheet",
    ".parameter CMD ...       Manage SQL parameter bindings",
    "   clear                   Erase all bindings",
    "   init                    Initialize the TEMP table that holds bindings",
    "   list                    List the current parameter bindings",
    "   set PARAMETER VALUE     Given SQL parameter PARAMETER a value of VALUE",
    "                           PARAMETER should start with one of: $ : @ ?",
    "   unset PARAMETER         Remove PARAMETER from the binding table",
    ".print STRING...         Print literal STRING",
    ".progress N              Invoke progress handler after every N opcodes",
    "   --limit N                 Interrupt after N progress callbacks",
    "   --once                    Do no more than one progress interrupt",
    "   --quiet|-q                No output except at interrupts",
    "   --reset                   Reset the count for each input and interrupt",
    ".prompt MAIN CONTINUE    Replace the standard prompts",
    ".quit                    Exit this program",
    ".read FILE               Read input from FILE or command output",
    "    If FILE begins with \"|\", it is a command that generates the input.",
    ".restore ?DB? FILE       Restore content of DB (default \"main\") from FILE",
    ".save ?OPTIONS? FILE     Write database to FILE (an alias for .backup ...)",
    ".scanstats on|off        Turn sqlite3_stmt_scanstatus() metrics on or off",
    ".schema ?PATTERN?        Show the CREATE statements matching PATTERN",
    "   Options:",
    "      --indent             Try to pretty-print the schema",
    "      --nosys              Omit objects whose names start with \"sqlite_\"",
    ".selftest ?OPTIONS?      Run tests defined in the SELFTEST table",
    "    Options:",
    "       --init               Create a new SELFTEST table",
    "       -v                   Verbose output",
    ".separator COL ?ROW?     Change the column and row separators",
    ".sha3sum ...             Compute a SHA3 hash of database content",
    "    Options:",
    "      --schema              Also hash the sqlite_schema table",
    "      --sha3-224            Use the sha3-224 algorithm",
    "      --sha3-256            Use the sha3-256 algorithm (default)",
    "      --sha3-384            Use the sha3-384 algorithm",
    "      --sha3-512            Use the sha3-512 algorithm",
    "    Any other argument is a LIKE pattern for tables to hash",
    ".shell CMD ARGS...       Run CMD ARGS... in a system shell",
    ".show                    Show the current values for various settings",
    ".stats ?ARG?             Show stats or turn stats on or off",
    "   off                      Turn off automatic stat display",
    "   on                       Turn on automatic stat display",
    "   stmt                     Show statement stats",
    "   vmstep                   Show the virtual machine step count only",
    ".system CMD ARGS...      Run CMD ARGS... in a system shell",
    ".tables ?TABLE?          List names of tables matching LIKE pattern TABLE",
    ".testcase NAME           Begin redirecting output to 'testcase-out.txt'",
    ".testctrl CMD ...        Run various sqlite3_test_control() operations",
    "                           Run \".testctrl\" with no arguments for details",
    ".timeout MS              Try opening locked tables for MS milliseconds",
    ".timer on|off            Turn SQL timer on or off",
    ".trace ?OPTIONS?         Output each SQL statement as it is run",
    "    FILE                    Send output to FILE",
    "    stdout                  Send output to stdout",
    "    stderr                  Send output to stderr",
    "    off                     Disable tracing",
    "    --expanded              Expand query parameters",
    "    --plain                 Show SQL as it is input",
    "    --stmt                  Trace statement execution (SQLITE_TRACE_STMT)",
    "    --profile               Profile statements (SQLITE_TRACE_PROFILE)",
    "    --row                   Trace each row (SQLITE_TRACE_ROW)",
    "    --close                 Trace connection close (SQLITE_TRACE_CLOSE)",
    ".vfsinfo ?AUX?           Information about the top-level VFS",
    ".vfslist                 List all available VFSes",
    ".vfsname ?AUX?           Print the name of the VFS stack",
    ".width NUM1 NUM2 ...     Set minimum column widths for columnar output",
    "     Negative values right-justify",
];

fn showHelp(zPattern: &str) -> i32 {
    let mut n: i32 = 0;

    if zPattern == "" || zPattern == "-a" || zPattern == "-all" || zPattern == "--all" {
        let all = if zPattern == "" { false } else { true };

        for help in azHelp {
            if help.starts_with('.') || all == true {
                println!("{}", help);
                n += 1;
            }
        }

        return n;
    }

    let mut i: usize = 0;
    let mut j: usize = 0;

    let zPat = format!(".{}", zPattern);
    for help in azHelp {
        if sqrite::strglob(&zPat, help) {
            println!("{}", help);
            j = i + 1;
            n += 1;
        }
        i += 1;
    }

    if n != 0 {
        if n == 1 {
            // print next line if only printed one line as long as not started with '.'
            while j < azHelp.len() && azHelp[j].starts_with('.') == false {
                println!("{}", azHelp[j]);
                j += 1;
            }
        }
        return n;
    }

    let zPat = format!("%{}%", zPattern);
    i = 0;
    while i < azHelp.len() {
        if azHelp[i].starts_with('.') {
            j = i;
        }

        if sqrite::strlike(&zPat, &azHelp[i], '\0') {
            println!("{}", azHelp[j]);
            while j < (176 - 1) && azHelp[j + 1].starts_with('.') == false {
                j += 1;
                println!("{}", azHelp[j]);
            }
            i = j;
            n += 1;
        }

        i += 1;
    }
    return n;
}

fn readFile(zName: &str) -> Vec<u8> {
    let mut in_0 = match File::open(zName) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };

    let mut buf = Vec::new();
    match in_0.read_to_end(&mut buf) {
        Ok(n_bytes) => {
            return buf;
        }
        Err(_) => return Vec::new(),
    }
}

fn deduceDatabaseType(zName: &str, dfltZip: i32) -> i32 {
    // let mut n: size_t = 0;
    let mut rc: i32 = 0;
    let mut zBuf: [i8; 100] = [0; 100];

    let mut f = match File::open(zName) {
        Err(_) => {
            if dfltZip != 0 && sqrite::strlike("%.zip", zName, '\0') {
                return 3;
            } else {
                return 1;
            }
        }
        Ok(f) => f,
    };

    let mut bufHeader = [0; 16];
    f.read_exact(&mut bufHeader).unwrap();
    if &bufHeader == b"SQLite format 3\0" {
        return 1;
    }

    f.seek(SeekFrom::End(25)).unwrap();
    let mut bufEnd = [0; 25];
    f.read_exact(&mut bufEnd).unwrap();
    let search = memmem::TwoWaySearcher::new(b"Start-Of-SQLite3-");

    if search.search_in(&bufEnd).is_some() {
        rc = 2;
    } else {
        f.seek(SeekFrom::End(22)).unwrap();
        let mut bufZip = [0; 22];
        match f.read_exact(&mut bufZip) {
            Ok(_) => {
                if zBuf[0] == 0x50 && zBuf[1] == 0x4b && zBuf[2] == 0x5 && zBuf[3] == 0x6 {
                    rc = 3;
                }
            }
            Err(_) => {
                if dfltZip != 0 && sqrite::strlike("%.zip", &zName, '\0') {
                    rc = 3;
                }
            }
        }
    }
    return rc;
}
unsafe fn readHexDb(mut p: *mut ShellState) -> Vec<u8> {
    let mut nLine: i32 = 0;
    let mut iOffset: i32 = 0;
    let mut k: i32 = 0;
    let mut zDbFilename = (*(*p).pAuxDb).zDbFilename.as_str();
    let mut restore_lineno = false;

    let stdin_ = std::io::stdin();
    let stdin_ = stdin_.lock();

    let mut reader = if zDbFilename.is_empty() == false {
        nLine = 0;
        let file = match File::open(zDbFilename) {
            Ok(x) => x,
            Err(_) => {
                eprintln!("cannot open \"{}\" for reading", zDbFilename);
                return vec![];
            }
        };
        Box::new(BufReader::new(file))
    } else {
        nLine = (*p).lineno;
        restore_lineno = true;

        Box::new(stdin_) as Box<dyn BufRead>
    };

    nLine += 1;

    let mut zLine = String::new();
    match reader.read_line(&mut zLine) {
        Err(_) => {}
        Ok(n) => {
            let parsed = sscanf::scanf!(zLine, "| size {i32} pagesize {i32}");
            match parsed {
                Err(_) => {}
                Ok((n, pgsz)) => {
                    if !(n < 0) {
                        if !(pgsz < 512 || pgsz > 65536 || pgsz & pgsz - 1 != 0) {
                            let n = n + pgsz - 1 & !(pgsz - 1);

                            let mut a: Vec<u8> = Vec::new();
                            a.reserve(if n != 0 { n } else { 1 } as usize);

                            if pgsz < 512 || pgsz > 65536 || pgsz & pgsz - 1 != 0 {
                                eprintln!("invalid pagesize");
                            } else {
                                nLine += 1;

                                while let Ok(n_bytes) = reader.read_line(&mut zLine) {
                                    let parsed = sscanf::scanf!(zLine, "| page {i32} offset {i32}");
                                    match parsed {
                                        Ok((j, k)) => {
                                            iOffset = k;
                                        }
                                        Err(_) => {
                                            if zLine == "| end " {
                                                break;
                                            }

                                            let parsed = sscanf::scanf!(zLine, "| {i32}: {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x} {u32:x}");
                                            match parsed {
                                                Err(_) => {}
                                                Ok((j, x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15)) => {
                                                    k = iOffset + j;
                                                    if k + 16 <= n && k >= 0 {
                                                        a[(k + 0) as usize] = (x0 & 0xff as i32 as u32) as u8;
                                                        a[(k + 1) as usize] = (x1 & 0xff as i32 as u32) as u8;
                                                        a[(k + 2) as usize] = (x2 & 0xff as i32 as u32) as u8;
                                                        a[(k + 3) as usize] = (x3 & 0xff as i32 as u32) as u8;
                                                        a[(k + 4) as usize] = (x4 & 0xff as i32 as u32) as u8;
                                                        a[(k + 5) as usize] = (x5 & 0xff as i32 as u32) as u8;
                                                        a[(k + 6) as usize] = (x6 & 0xff as i32 as u32) as u8;
                                                        a[(k + 7) as usize] = (x7 & 0xff as i32 as u32) as u8;
                                                        a[(k + 8) as usize] = (x8 & 0xff as i32 as u32) as u8;
                                                        a[(k + 9) as usize] = (x9 & 0xff as i32 as u32) as u8;
                                                        a[(k + 10) as usize] = (x10 & 0xff as i32 as u32) as u8;
                                                        a[(k + 11) as usize] = (x11 & 0xff as i32 as u32) as u8;
                                                        a[(k + 12) as usize] = (x12 & 0xff as i32 as u32) as u8;
                                                        a[(k + 13) as usize] = (x13 & 0xff as i32 as u32) as u8;
                                                        a[(k + 14) as usize] = (x14 & 0xff as i32 as u32) as u8;
                                                        a[(k + 15) as usize] = (x15 & 0xff as i32 as u32) as u8;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    nLine += 1;
                                }

                                if restore_lineno {
                                    (*p).lineno = nLine;
                                }
                                return a;
                            }
                        }
                    }
                }
            }
        }
    }

    if restore_lineno {
        while let Ok(_) = reader.read_line(&mut zLine) {
            nLine += 1;
            if zLine == "| end " {
                break;
            }
        }
        (*p).lineno = nLine;
    }

    eprintln!("Error on line {} of --hexdb input", nLine);
    return Vec::new();
}

unsafe extern "C" fn shellInt32(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut pBlob: *const u8 = std::ptr::null();
    let mut nBlob: i32 = 0;
    let mut iInt: i32 = 0;
    nBlob = sqlite3_value_bytes(*argv.offset(0));
    pBlob = sqlite3_value_blob(*argv.offset(0)) as *const u8;
    iInt = sqlite3_value_int(*argv.offset(1));
    if iInt >= 0 && (iInt + 1) * 4 <= nBlob {
        let mut a: *const u8 = &*pBlob.offset((iInt * 4) as isize) as *const u8;
        let mut iVal: i64 = ((*a.offset(0) as i64) << 24)
            + ((*a.offset(1) as i64) << 16)
            + ((*a.offset(2 as isize) as i64) << 8)
            + ((*a.offset(3 as isize) as i64) << 0);
        sqlite3_result_int64(context, iVal);
    }
}
unsafe extern "C" fn shellIdQuote(mut context: *mut sqlite3_context, mut argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut zName: *const i8 = sqlite3_value_text(*argv.offset(0)) as *const i8;
    if !zName.is_null() {
        let mut z: *mut i8 = sqlite3_mprintf(b"\"%w\"\0" as *const u8 as *const i8, zName);
        sqlite3_result_text(context, z, -1, Some(sqlite3_free));
    }
}
unsafe extern "C" fn shellUSleepFunc(mut context: *mut sqlite3_context, mut _argcUnused: i32, mut argv: *mut *mut sqlite3_value) {
    let mut sleep: i32 = sqlite3_value_int(*argv.offset(0));
    sqlite3_sleep(sleep / 1000);
    sqlite3_result_int(context, sleep);
}
unsafe extern "C" fn shellEscapeCrnl(mut context: *mut sqlite3_context, mut _argc: i32, mut argv: *mut *mut sqlite3_value) {
    let mut zText: *const i8 = sqlite3_value_text(*argv.offset(0)) as *const i8;
    if !zText.is_null() && *zText.offset(0) as i32 == '\'' as i32 {
        let mut nText: i32 = sqlite3_value_bytes(*argv.offset(0));
        let mut zBuf1 = String::new();
        let mut zBuf2 = String::new();
        let mut zNL = "";
        let mut zCR = "";
        let mut nCR: i32 = 0;
        let mut nNL: i32 = 0;
        let rzText = cstr_to_string!(zText);
        let rzText = rzText.as_bytes();

        for i in 0..rzText.len() {
            if zNL.is_empty() && rzText[i] == b'\n' {
                zNL = unused_string(&rzText, "\\n", "\\012", &mut zBuf1);
                nNL = zNL.len() as i32;
            }
            if zCR.is_empty() && rzText[i] == b'\r' {
                zCR = unused_string(&rzText, "\\r", "\\015", &mut zBuf2);
                nCR = zCR.len() as i32;
            }
        }

        if !zNL.is_empty() || !zCR.is_empty() {
            let nMax = (if nNL > nCR { nNL } else { nCR }) as usize;
            let nAlloc = nMax * nText as usize + (nMax + 64) * 2;
            let mut zOut = ShellText::new_with_capacity(nAlloc as usize);
            let mut iOut = 0;
            if !zNL.is_empty() && !zCR.is_empty() {
                zOut.push("replace(replace(");
                iOut += 16;
            } else {
                zOut.push("replace(");
                iOut += 8;
            }

            for i in 0..rzText.len() {
                if rzText[i] == b'\n' {
                    zOut.push(zNL);
                    iOut += nNL;
                } else if rzText[i] == b'\r' {
                    zOut.push(zCR);
                    iOut += nCR;
                } else {
                    zOut.push_u8(rzText[i]);
                    iOut += 1;
                }
            }

            if !zNL.is_empty() {
                zOut.push(",'");
                iOut += 2;
                zOut.push(zNL);
                iOut += nNL;
                zOut.push("', char(10))");
                iOut += 12;
            }
            if !zCR.is_empty() {
                zOut.push(",'");
                iOut += 2;
                zOut.push(zCR);
                iOut += nCR;
                zOut.push("', char(13))");
                iOut += 12;
            }

            sqlite3_result_text(
                context,
                zOut.to_cstr(),
                iOut,
                ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
            );
            return;
        }
    }
    sqlite3_result_value(context, *argv.offset(0));
}

unsafe fn open_db(p: *mut ShellState, openFlags: i32) {
    if ((*p).db).is_null() {
        let zDbFilename = (*(*p).pAuxDb).zDbFilename.as_str();
        if (*p).openMode as i32 == 0 {
            if zDbFilename.is_empty() {
                (*p).openMode = 1;
            } else {
                (*p).openMode = deduceDatabaseType(zDbFilename, (openFlags & 0x2 != 0) as i32) as u8;
            }
        }
        match (*p).openMode as i32 {
            2 => {
                sqlite3_open_v2(
                    to_cstring!(zDbFilename),
                    &mut (*p).db,
                    0x2 | 0x4 as i32 | (*p).openFlags,
                    b"apndvfs\0" as *const u8 as *const i8,
                );
            }
            6 | 5 => {
                sqlite3_open(0 as *const i8, &mut (*p).db);
            }
            3 => {
                sqlite3_open(b":memory:\0" as *const u8 as *const i8, &mut (*p).db);
            }
            4 => {
                sqlite3_open_v2(to_cstring!(zDbFilename), &mut (*p).db, 0x1 | (*p).openFlags, 0 as *const i8);
            }
            0 | 1 => {
                sqlite3_open_v2(to_cstring!(zDbFilename), &mut (*p).db, 0x2 | 0x4 | (*p).openFlags, 0 as *const i8);
            }
            _ => {}
        }
        globalDb = (*p).db;
        if ((*p).db).is_null() || 0 != sqlite3_errcode((*p).db) {
            eprintln!(
                "Error: unable to open database \"{}\": {}",
                zDbFilename,
                cstr_to_string!(sqlite3_errmsg((*p).db))
            );
            if openFlags & 0x1 != 0 {
                sqlite3_open(b":memory:\0" as *const u8 as *const i8, &mut (*p).db);
                return;
            }
            std::process::exit(1);
        }
        sqlite3_enable_load_extension((*p).db, 1);
        sqlite3_fileio_init((*p).db, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
        sqlite3_shathree_init((*p).db, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
        sqlite3_completion_init((*p).db, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
        sqlite3_uint_init((*p).db, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
        sqlite3_decimal_init((*p).db, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
        sqlite3_regexp_init((*p).db, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
        sqlite3_ieee_init((*p).db, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
        sqlite3_series_init((*p).db, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
        sqlite3_create_function(
            (*p).db,
            b"shell_add_schema\0" as *const u8 as *const i8,
            3,
            1,
            std::ptr::null_mut(),
            Some(shellAddSchemaName),
            None,
            None,
        );
        sqlite3_create_function(
            (*p).db,
            b"shell_module_schema\0" as *const u8 as *const i8,
            1,
            1,
            std::ptr::null_mut(),
            Some(shellModuleSchema),
            None,
            None,
        );
        sqlite3_create_function(
            (*p).db,
            b"shell_putsnl\0" as *const u8 as *const i8,
            1,
            1,
            p as *mut libc::c_void,
            Some(shellPutsFunc),
            None,
            None,
        );
        sqlite3_create_function(
            (*p).db,
            b"shell_escape_crnl\0" as *const u8 as *const i8,
            1,
            1,
            std::ptr::null_mut(),
            Some(shellEscapeCrnl),
            None,
            None,
        );
        sqlite3_create_function(
            (*p).db,
            b"shell_int32\0" as *const u8 as *const i8,
            2,
            1,
            std::ptr::null_mut(),
            Some(shellInt32),
            None,
            None,
        );
        sqlite3_create_function(
            (*p).db,
            b"shell_idquote\0" as *const u8 as *const i8,
            1,
            1,
            std::ptr::null_mut(),
            Some(shellIdQuote),
            None,
            None,
        );
        sqlite3_create_function(
            (*p).db,
            b"usleep\0" as *const u8 as *const i8,
            1,
            1,
            std::ptr::null_mut(),
            Some(shellUSleepFunc),
            None,
            None,
        );
        sqlite3_create_function(
            (*p).db,
            b"edit\0" as *const u8 as *const i8,
            1,
            1,
            std::ptr::null_mut(),
            Some(editFunc),
            None,
            None,
        );
        sqlite3_create_function(
            (*p).db,
            b"edit\0" as *const u8 as *const i8,
            2,
            1,
            std::ptr::null_mut(),
            Some(editFunc),
            None,
            None,
        );
        if (*p).openMode == 3 {
            let mut zSql: *mut i8 = sqlite3_mprintf(
                b"CREATE VIRTUAL TABLE zip USING zipfile(%Q);\0" as *const u8 as *const i8,
                zDbFilename,
            );
            shell_check_oom(zSql as *mut libc::c_void);
            sqlite3_exec((*p).db, zSql, None, std::ptr::null_mut(), 0 as *mut *mut i8);
            sqlite3_free(zSql as *mut libc::c_void);
        } else if (*p).openMode == 5 || (*p).openMode == 6 {
            let mut aData = if (*p).openMode == 5 {
                readFile(zDbFilename)
            } else {
                let aData = readHexDb(p);
                if aData.is_empty() {
                    return;
                }
                aData
            };

            let rc = sqlite3_deserialize(
                (*p).db,
                b"main\0" as *const u8 as *const i8,
                aData.as_mut_ptr(),
                aData.len() as i64,
                aData.len() as i64,
                (2 | 1) as u32,
            );
            if rc != 0 {
                eprintln!("Error: sqlite3_deserialize() returns {rc}");
            }
            if (*p).szMax > 0 {
                sqlite3_file_control(
                    (*p).db,
                    b"main\0" as *const u8 as *const i8,
                    36,
                    &mut (*p).szMax as *mut i64 as *mut libc::c_void,
                );
            }
        }
    }
    if (*p).bSafeModePersist as i32 != 0 && !((*p).db).is_null() {
        sqlite3_set_authorizer((*p).db, Some(safeModeAuth), p as *mut libc::c_void);
    }
}

unsafe fn close_db(mut db: *mut sqlite3) {
    let mut rc: i32 = sqlite3_close(db);
    if rc != 0 {
        eprintln!("Error: sqlite3_close() returns {}: {}", rc, cstr_to_string!(sqlite3_errmsg(db)));
    }
}

/*
** Do C-language style dequoting.
**
**    \a    -> alarm
**    \b    -> backspace
**    \t    -> tab
**    \n    -> newline
**    \v    -> vertical tab
**    \f    -> form feed
**    \r    -> carriage return
**    \s    -> space
**    \"    -> "
**    \'    -> '
**    \\    -> backslash
**    \NNN  -> ascii character NNN in octal
*/
fn resolve_backslashes(z: &str) -> String {
    let mut buf = String::new();
    buf.reserve(z.len());

    let mut iters = z.chars();
    while let Some(ch) = iters.next() {
        match ch {
            '\\' => {
                let c = match iters.next() {
                    None => {
                        buf.push(ch);
                        break;
                    }
                    Some(ch2) => match ch2 {
                        // 'a' => '\a',
                        // 'b' => '\b',
                        't' => '\t',
                        'n' => '\n',
                        // 'v' => '\v',
                        // 'f' => '\f',
                        'r' => '\r',
                        '"' => '"',
                        '\'' => '\'',
                        '\\' => '\\',
                        '0'..='7' => {
                            let mut n = ch2 as u8 - '0' as u8;
                            let ch3 = iters.next();
                            match ch3 {
                                None => {
                                    buf.push(ch);
                                    buf.push(ch2);
                                    break;
                                }
                                Some(ch3) => match ch3 {
                                    '0'..='7' => {
                                        n = (n << 3) + (ch3 as u8 - '0' as u8);
                                        let ch4 = iters.next();
                                        match ch4 {
                                            None => {
                                                buf.push(ch2);
                                                buf.push(ch3);
                                                break;
                                            }
                                            Some(ch4) => match ch4 {
                                                '0'..='7' => {
                                                    n = (n << 3) + (ch4 as u8 - '0' as u8);
                                                }
                                                _ => {
                                                    buf.push(ch2);
                                                    buf.push(ch3);
                                                    buf.push(ch4);
                                                    break;
                                                }
                                            },
                                        }
                                    }
                                    _ => {
                                        buf.push(ch2);
                                        buf.push(ch3);
                                        break;
                                    }
                                },
                            };

                            n as char
                        }
                        _ => {
                            buf.push(ch2);
                            break;
                        }
                    },
                };
                buf.push(c);
            }
            c => buf.push(c),
        }
    }

    return buf;
}

fn booleanValue(zArg: &str) -> i32 {
    let mut i: usize = 0;

    if zArg.starts_with("0x") {
        i = 2;
        for ch in zArg[2..].chars() {
            if ch.is_ascii_hexdigit() {
                i += 1;
            }
        }
    } else {
        i = 0;
        for ch in zArg.chars() {
            if ch.is_ascii_digit() {
                i += 1;
            }
        }
    }

    if i > 0 && i == zArg.len() {
        return (integerValue(&zArg) & 0xffffffff as i64) as i32;
    }

    match zArg.to_ascii_lowercase().as_str() {
        "on" | "yes" => return 1,
        "off" | "no" => return 0,
        _ => {
            eprintln!("ERROR: Not a boolean value: \"{}\". Assuming \"no\".", zArg);
            return 0;
        }
    }
}

fn setOrClearFlag(p: &mut ShellState, mFlag: u32, zArg: &str) {
    if booleanValue(zArg) != 0 {
        p.shellFlgs |= mFlag;
    } else {
        p.shellFlgs &= !mFlag;
    };
}

unsafe fn output_file_close(f: *mut FILE) {
    if !f.is_null() && f != stdout && f != stderr {
        fclose(f);
    }
}
unsafe fn output_file_open(zFile: &str, bTextMode: i32) -> *mut FILE {
    let mut f: *mut FILE = 0 as *mut FILE;
    if zFile == "stdout" {
        f = stdout;
    } else if zFile == "stderr" {
        f = stderr;
    } else if zFile == "off" {
        f = 0 as *mut FILE;
    } else {
        f = fopen(
            to_cstring!(zFile),
            if bTextMode != 0 {
                b"w\0" as *const u8 as *const i8
            } else {
                b"wb\0" as *const u8 as *const i8
            },
        );
        if f.is_null() {
            eprintln!("Error: cannot open \"{}\"", zFile);
        }
    }
    return f;
}
unsafe extern "C" fn sql_trace_callback(
    mut mType: u32,
    mut pArg: *mut libc::c_void,
    mut pP: *mut libc::c_void,
    mut pX: *mut libc::c_void,
) -> i32 {
    let mut p: *mut ShellState = pArg as *mut ShellState;
    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut zSql: *const i8 = 0 as *const i8;
    let mut nSql: i32 = 0;
    if ((*p).traceOut).is_null() {
        return 0;
    }
    if mType == 0x8 {
        fprintf((*p).traceOut, b"-- closing database connection\n\0" as *const u8 as *const i8);
        return 0;
    }
    if mType != 0x4 as i32 as u32 && *(pX as *const i8).offset(0) as i32 == '-' as i32 {
        zSql = pX as *const i8;
    } else {
        pStmt = pP as *mut sqlite3_stmt;
        match (*p).eTraceType as i32 {
            1 => {
                zSql = sqlite3_expanded_sql(pStmt);
            }
            _ => {
                zSql = sqlite3_sql(pStmt);
            }
        }
    }
    if zSql.is_null() {
        return 0;
    }
    nSql = strlen30(zSql);
    while nSql > 0 && *zSql.offset((nSql - 1) as isize) as i32 == ';' as i32 {
        nSql -= 1;
    }
    match mType {
        4 | 1 => {
            fprintf((*p).traceOut, b"%.*s;\n\0" as *const u8 as *const i8, nSql, zSql);
        }
        2 => {
            let nNanosec: i64 = *(pX as *mut i64);
            fprintf(
                (*p).traceOut,
                b"%.*s; -- %lld ns\n\0" as *const u8 as *const i8,
                nSql,
                zSql,
                nNanosec,
            );
        }
        _ => {}
    }
    return 0;
}
unsafe fn test_breakpoint() {
    static mut nCall: i32 = 0;
    nCall += 1;
}
unsafe fn import_cleanup(p: *mut ImportCtx) {
    if !((*p).in_0).is_null() && ((*p).xCloser).is_some() {
        ((*p).xCloser).expect("non-null function pointer")((*p).in_0);
        (*p).in_0 = 0 as *mut FILE;
    }
    sqlite3_free((*p).z as *mut libc::c_void);
    (*p).z = std::ptr::null_mut();
}

unsafe fn import_append_char(p: *mut ImportCtx, c: i32) {
    if (*p).n + 1 >= (*p).nAlloc {
        (*p).nAlloc += (*p).nAlloc + 100;
        (*p).z = sqlite3_realloc64((*p).z as *mut libc::c_void, (*p).nAlloc as u64) as *mut i8;
        shell_check_oom((*p).z as *mut libc::c_void);
    }
    *((*p).z).offset((*p).n as isize) = c as i8;
    (*p).n += 1;
}

unsafe extern "C" fn csv_read_one_field(p: *mut ImportCtx) -> *mut i8 {
    let mut c: i32 = 0;
    let mut cSep: i32 = (*p).cColSep;
    let mut rSep: i32 = (*p).cRowSep;
    (*p).n = 0;
    c = fgetc((*p).in_0);
    if c == -1 || seenInterrupt != 0 {
        (*p).cTerm = -1;
        return std::ptr::null_mut();
    }
    if c == '"' as i32 {
        let mut pc: i32 = 0;
        let mut ppc: i32 = 0;
        let mut startLine: i32 = (*p).nLine;
        let mut cQuote: i32 = c;
        ppc = 0;
        pc = ppc;
        loop {
            c = fgetc((*p).in_0);
            if c == rSep {
                (*p).nLine += 1;
            }
            if c == cQuote {
                if pc == cQuote {
                    pc = 0;
                    continue;
                }
            }
            if c == cSep && pc == cQuote
                || c == rSep && pc == cQuote
                || c == rSep && pc == '\r' as i32 && ppc == cQuote
                || c == -1 && pc == cQuote
            {
                loop {
                    (*p).n -= 1;
                    if !(*((*p).z).offset((*p).n as isize) as i32 != cQuote) {
                        break;
                    }
                }
                (*p).cTerm = c;
                break;
            } else {
                if pc == cQuote && c != '\r' as i32 {
                    eprintln!("{}:{}: unescaped {} character", cstr_to_string!((*p).zFile), (*p).nLine, cQuote);
                }
                if c == -1 {
                    eprintln!(
                        "{}:{}: unterminated {}-quoted field",
                        cstr_to_string!((*p).zFile),
                        startLine,
                        cQuote
                    );
                    (*p).cTerm = c;
                    break;
                } else {
                    import_append_char(p, c);
                    ppc = pc;
                    pc = c;
                }
            }
        }
    } else {
        if c & 0xff as i32 == 0xef as i32 && (*p).bNotFirst == 0 {
            import_append_char(p, c);
            c = fgetc((*p).in_0);
            if c & 0xff as i32 == 0xbb as i32 {
                import_append_char(p, c);
                c = fgetc((*p).in_0);
                if c & 0xff as i32 == 0xbf as i32 {
                    (*p).bNotFirst = 1;
                    (*p).n = 0;
                    return csv_read_one_field(p);
                }
            }
        }
        while c != -1 && c != cSep && c != rSep {
            import_append_char(p, c);
            c = fgetc((*p).in_0);
        }
        if c == rSep {
            (*p).nLine += 1;
            if (*p).n > 0 && *((*p).z).offset(((*p).n - 1) as isize) as i32 == '\r' as i32 {
                (*p).n -= 1;
            }
        }
        (*p).cTerm = c;
    }
    if !((*p).z).is_null() {
        *((*p).z).offset((*p).n as isize) = 0;
    }
    (*p).bNotFirst = 1;
    return (*p).z;
}

unsafe extern "C" fn ascii_read_one_field(p: *mut ImportCtx) -> *mut i8 {
    let mut c: i32 = 0;
    let mut cSep: i32 = (*p).cColSep;
    let mut rSep: i32 = (*p).cRowSep;
    (*p).n = 0;
    c = fgetc((*p).in_0);
    if c == -1 || seenInterrupt != 0 {
        (*p).cTerm = -1;
        return std::ptr::null_mut();
    }
    while c != -1 && c != cSep && c != rSep {
        import_append_char(p, c);
        c = fgetc((*p).in_0);
    }
    if c == rSep {
        (*p).nLine += 1;
    }
    (*p).cTerm = c;
    if !((*p).z).is_null() {
        *((*p).z).offset((*p).n as isize) = 0;
    }
    return (*p).z;
}

unsafe extern "C" fn tryToCloneData(p: *mut ShellState, newDb: *mut sqlite3, zTable: *const i8) {
    let mut pQuery: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut pInsert: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut zQuery: *mut i8 = std::ptr::null_mut();
    let mut zInsert: *mut i8 = std::ptr::null_mut();
    let mut rc: i32 = 0;
    let mut i: i32 = 0;
    let mut n: i32 = 0;
    let mut nTable: i32 = strlen30(zTable);
    let mut cnt: i32 = 0;
    let spinRate: i32 = 10000;
    zQuery = sqlite3_mprintf(b"SELECT * FROM \"%w\"\0" as *const u8 as *const i8, zTable);
    shell_check_oom(zQuery as *mut libc::c_void);
    rc = sqlite3_prepare_v2((*p).db, zQuery, -1, &mut pQuery, 0 as *mut *const i8);
    if rc != 0 {
        eprintln!(
            "Error {}: {} on [{}]",
            sqlite3_extended_errcode((*p).db),
            cstr_to_string!(sqlite3_errmsg((*p).db)),
            cstr_to_string!(zQuery),
        );
    } else {
        n = sqlite3_column_count(pQuery);
        zInsert = sqlite3_malloc64((200 + nTable + n * 3) as u64) as *mut i8;
        shell_check_oom(zInsert as *mut libc::c_void);
        sqlite3_snprintf(
            200 + nTable,
            zInsert,
            b"INSERT OR IGNORE INTO \"%s\" VALUES(?\0" as *const u8 as *const i8,
            zTable,
        );
        i = strlen30(zInsert);
        for _ in 1..n {
            memcpy(
                zInsert.offset(i as isize) as *mut libc::c_void,
                b",?\0" as *const u8 as *const i8 as *const libc::c_void,
                2 as u64,
            );
            i += 2;
        }
        memcpy(
            zInsert.offset(i as isize) as *mut libc::c_void,
            b");\0" as *const u8 as *const i8 as *const libc::c_void,
            3 as u64,
        );
        rc = sqlite3_prepare_v2(newDb, zInsert, -1, &mut pInsert, 0 as *mut *const i8);
        if rc != 0 {
            eprintln!(
                "Error {}: {} on [{}]",
                sqlite3_extended_errcode(newDb),
                cstr_to_string!(sqlite3_errmsg(newDb)),
                cstr_to_string!(zQuery),
            );
        } else {
            for _ in 0..2 {
                loop {
                    rc = sqlite3_step(pQuery);
                    if !(rc == 100) {
                        break;
                    }
                    for i in 0..n {
                        match sqlite3_column_type(pQuery, i) {
                            5 => {
                                sqlite3_bind_null(pInsert, i + 1);
                            }
                            1 => {
                                sqlite3_bind_int64(pInsert, i + 1, sqlite3_column_int64(pQuery, i));
                            }
                            2 => {
                                sqlite3_bind_double(pInsert, i + 1, sqlite3_column_double(pQuery, i));
                            }
                            3 => {
                                sqlite3_bind_text(pInsert, i + 1, sqlite3_column_text(pQuery, i) as *const i8, -1, None);
                            }
                            4 => {
                                sqlite3_bind_blob(
                                    pInsert,
                                    i + 1,
                                    sqlite3_column_blob(pQuery, i),
                                    sqlite3_column_bytes(pQuery, i),
                                    None,
                                );
                            }
                            _ => {}
                        }
                    }
                    rc = sqlite3_step(pInsert);
                    if rc != 0 && rc != 100 && rc != 101 {
                        eprintln!(
                            "Error {}: {}",
                            sqlite3_extended_errcode(newDb),
                            cstr_to_string!(sqlite3_errmsg(newDb)),
                        );
                    }
                    sqlite3_reset(pInsert);
                    cnt += 1;
                    if cnt % spinRate == 0 {
                        print!("{}\x08", b"|/-\\"[(cnt / spinRate % 4) as usize] as char);
                        fflush(stdout);
                    }
                }
                if rc == 101 {
                    break;
                }
                sqlite3_finalize(pQuery);
                sqlite3_free(zQuery as *mut libc::c_void);
                zQuery = sqlite3_mprintf(b"SELECT * FROM \"%w\" ORDER BY rowid DESC;\0" as *const u8 as *const i8, zTable);
                shell_check_oom(zQuery as *mut libc::c_void);
                rc = sqlite3_prepare_v2((*p).db, zQuery, -1, &mut pQuery, 0 as *mut *const i8);
                if rc != 0 {
                    eprintln!("Warning: cannot step \"{}\" backwards", cstr_to_string!(zTable));
                    break;
                }
            }
        }
    }
    sqlite3_finalize(pQuery);
    sqlite3_finalize(pInsert);
    sqlite3_free(zQuery as *mut libc::c_void);
    sqlite3_free(zInsert as *mut libc::c_void);
}

/*
** Try to transfer all rows of the schema that match zWhere.  For
** each row, invoke xForEach() on the object defined by that row.
** If an error is encountered while moving forward through the
** sqlite_schema table, try again moving backwards.
*/
unsafe fn tryToCloneSchema(
    mut p: *mut ShellState,
    mut newDb: *mut sqlite3,
    zWhere: &str,
    mut xForEach: Option<unsafe extern "C" fn(*mut ShellState, *mut sqlite3, *const i8) -> ()>,
) {
    let mut pQuery: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut zQuery: *mut i8 = std::ptr::null_mut();
    let mut rc: i32 = 0;
    let mut zName: *const u8 = std::ptr::null();
    let mut zSql: *const u8 = std::ptr::null();
    let mut zErrMsg: *mut i8 = std::ptr::null_mut();
    zQuery = sqlite3_mprintf(
        b"SELECT name, sql FROM sqlite_schema WHERE %s\0" as *const u8 as *const i8,
        to_cstring!(zWhere),
    );
    shell_check_oom(zQuery as *mut libc::c_void);
    rc = sqlite3_prepare_v2((*p).db, zQuery, -1, &mut pQuery, 0 as *mut *const i8);
    if rc != 0 {
        eprintln!(
            "Error: ({}) {} on [{}]",
            sqlite3_extended_errcode((*p).db),
            cstr_to_string!(sqlite3_errmsg((*p).db)),
            cstr_to_string!(zQuery)
        );
    } else {
        loop {
            rc = sqlite3_step(pQuery);
            if !(rc == 100) {
                break;
            }
            zName = sqlite3_column_text(pQuery, 0);
            zSql = sqlite3_column_text(pQuery, 1);
            if zName.is_null() || zSql.is_null() {
                continue;
            }
            print!("{}... ", cstr_to_string!(zName as *const i8));
            fflush(stdout);
            sqlite3_exec(newDb, zSql as *const i8, None, std::ptr::null_mut(), &mut zErrMsg);
            if !zErrMsg.is_null() {
                eprintln!("Error: {}\nSQL: [{}]", cstr_to_string!(zErrMsg), cstr_to_string!(zSql as *const i8));
                sqlite3_free(zErrMsg as *mut libc::c_void);
                zErrMsg = std::ptr::null_mut();
            }
            if xForEach.is_some() {
                xForEach.expect("non-null function pointer")(p, newDb, zName as *const i8);
            }
            println!("done");
        }
        if rc != 101 {
            sqlite3_finalize(pQuery);
            sqlite3_free(zQuery as *mut libc::c_void);
            zQuery = sqlite3_mprintf(
                b"SELECT name, sql FROM sqlite_schema WHERE %s ORDER BY rowid DESC\0" as *const u8 as *const i8,
                to_cstring!(zWhere),
            );
            shell_check_oom(zQuery as *mut libc::c_void);
            rc = sqlite3_prepare_v2((*p).db, zQuery, -1, &mut pQuery, 0 as *mut *const i8);
            if rc != 0 {
                eprintln!(
                    "Error: ({}) {} on [{}]",
                    sqlite3_extended_errcode((*p).db),
                    cstr_to_string!(sqlite3_errmsg((*p).db)),
                    cstr_to_string!(zQuery),
                );
            } else {
                while sqlite3_step(pQuery) == 100 {
                    zName = sqlite3_column_text(pQuery, 0);
                    zSql = sqlite3_column_text(pQuery, 1);
                    if zName.is_null() || zSql.is_null() {
                        continue;
                    }
                    print!("{}...", cstr_to_string!(zName as *const i8));
                    fflush(stdout);
                    sqlite3_exec(newDb, zSql as *const i8, None, std::ptr::null_mut(), &mut zErrMsg);
                    if !zErrMsg.is_null() {
                        eprintln!("Error: {}\nSQL: [{}]", cstr_to_string!(zErrMsg), cstr_to_string!(zSql as *const i8),);
                        sqlite3_free(zErrMsg as *mut libc::c_void);
                        zErrMsg = std::ptr::null_mut();
                    }
                    if xForEach.is_some() {
                        xForEach.expect("non-null function pointer")(p, newDb, zName as *const i8);
                    }
                    println!("done");
                }
            }
        }
    }
    sqlite3_finalize(pQuery);
    sqlite3_free(zQuery as *mut libc::c_void);
}

/*
** Open a new database file named "zNewDb".  Try to recover as much information
** as possible out of the main database (which might be corrupt) and write it
** into zNewDb.
*/

unsafe fn tryToClone(mut p: *mut ShellState, zNewDb: &str) {
    let mut rc: i32 = 0;
    let mut newDb: *mut sqlite3 = 0 as *mut sqlite3;
    if Path::new(zNewDb).exists() {
        eprintln!("File \"{}\" already exists.", zNewDb);
        return;
    }

    let czNewDb = to_cstring!(zNewDb);
    rc = sqlite3_open(czNewDb, &mut newDb);
    if rc != 0 {
        eprintln!("Cannot create output database: {}", cstr_to_string!(sqlite3_errmsg(newDb)));
    } else {
        sqlite3_exec(
            (*p).db,
            b"PRAGMA writable_schema=ON;\0" as *const u8 as *const i8,
            None,
            std::ptr::null_mut(),
            0 as *mut *mut i8,
        );
        sqlite3_exec(
            newDb,
            b"BEGIN EXCLUSIVE;\0" as *const u8 as *const i8,
            None,
            std::ptr::null_mut(),
            0 as *mut *mut i8,
        );
        tryToCloneSchema(p, newDb, "type='table'", Some(tryToCloneData));
        tryToCloneSchema(p, newDb, "type!='table'", None);
        sqlite3_exec(
            newDb,
            b"COMMIT;\0" as *const u8 as *const i8,
            None,
            std::ptr::null_mut(),
            0 as *mut *mut i8,
        );
        sqlite3_exec(
            (*p).db,
            b"PRAGMA writable_schema=OFF;\0" as *const u8 as *const i8,
            None,
            std::ptr::null_mut(),
            0 as *mut *mut i8,
        );
    }
    close_db(newDb);
}

unsafe fn output_reset(p: *mut ShellState) {
    if (*p).outfile[0] as i32 == '|' as i32 {
        // pclose((*p).out);
    } else {
        // output_file_close((*p).out);
        if (*p).doXdgOpen != 0 {
            let mut zXdgOpenCmd = "xdg-open";
            let mut cmd = std::process::Command::new(zXdgOpenCmd);
            cmd.arg(&(*p).zTempFile);

            match cmd.output() {
                Ok(_) => {
                    sqlite3_sleep(2000);
                }
                Err(_) => {
                    eprintln!("Failed: [{} {}]", zXdgOpenCmd, (*p).zTempFile);
                }
            }

            outputModePop(&mut *p);
            (*p).doXdgOpen = 0;
        }
    }
    (*p).outfile[0] = 0;
    // (*p).out = stdout;
}
unsafe extern "C" fn db_int(mut db: *mut sqlite3, mut zSql: *const i8) -> i32 {
    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut res: i32 = 0;
    sqlite3_prepare_v2(db, zSql, -1, &mut pStmt, 0 as *mut *const i8);
    if !pStmt.is_null() && sqlite3_step(pStmt) == 100 {
        res = sqlite3_column_int(pStmt, 0);
    }
    sqlite3_finalize(pStmt);
    return res;
}
fn get2byteInt(a: [u8; 2]) -> u32 {
    return (((a[0] as i32) << 8) + a[1] as i32) as u32;
}

fn get4byteInt(a: [u8; 4]) -> u32 {
    return (((a[0] as i32) << 24) + ((a[1] as i32) << 16) + ((a[2] as i32) << 8) + a[3] as i32) as u32;
}

unsafe fn shell_dbinfo_command(p: *mut ShellState, nArg: i32, razArg: &Vec<String>) -> i32 {
    const aField: [C2RustUnnamed_23; 12] = [
        C2RustUnnamed_23 {
            zName: "file change counter:",
            ofst: 24,
        },
        C2RustUnnamed_23 {
            zName: "database page count:",
            ofst: 28,
        },
        C2RustUnnamed_23 {
            zName: "freelist page count:",
            ofst: 36,
        },
        C2RustUnnamed_23 {
            zName: "schema cookie:",
            ofst: 40,
        },
        C2RustUnnamed_23 {
            zName: "schema format:",
            ofst: 44 as i32,
        },
        C2RustUnnamed_23 {
            zName: "default cache size:",
            ofst: 48,
        },
        C2RustUnnamed_23 {
            zName: "autovacuum top root:",
            ofst: 52,
        },
        C2RustUnnamed_23 {
            zName: "incremental vacuum:",
            ofst: 64 as i32,
        },
        C2RustUnnamed_23 {
            zName: "text encoding:",
            ofst: 56,
        },
        C2RustUnnamed_23 {
            zName: "user version:",
            ofst: 60,
        },
        C2RustUnnamed_23 {
            zName: "application id:",
            ofst: 68,
        },
        C2RustUnnamed_23 {
            zName: "software version:",
            ofst: 96,
        },
    ];
    const aQuery: [C2RustUnnamed_22; 5] = [
        C2RustUnnamed_22 {
            zName: "number of tables:",
            zSql: "SELECT count(*) FROM %s WHERE type='table'",
        },
        C2RustUnnamed_22 {
            zName: "number of indexes:",
            zSql: "SELECT count(*) FROM %s WHERE type='index'",
        },
        C2RustUnnamed_22 {
            zName: "number of triggers:",
            zSql: "SELECT count(*) FROM %s WHERE type='trigger'",
        },
        C2RustUnnamed_22 {
            zName: "number of views:",
            zSql: "SELECT count(*) FROM %s WHERE type='view'",
        },
        C2RustUnnamed_22 {
            zName: "schema size:",
            zSql: "SELECT total(length(sql)) FROM %s",
        },
    ];
    let mut i: i32 = 0;
    let mut rc: i32 = 0;
    let mut iDataVersion: u32 = 0;
    let mut zSchemaTab: *mut i8 = std::ptr::null_mut();
    let mut zDb = if nArg >= 2 { razArg[1].as_str() } else { "main" };
    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut aHdr: [u8; 100] = [0; 100];
    open_db(p, 0);
    if ((*p).db).is_null() {
        return 1;
    }
    rc = sqlite3_prepare_v2(
        (*p).db,
        b"SELECT data FROM sqlite_dbpage(?1) WHERE pgno=1\0" as *const u8 as *const i8,
        -1,
        &mut pStmt,
        0 as *mut *const i8,
    );
    if rc != 0 {
        eprintln!("error: {}", cstr_to_string!(sqlite3_errmsg((*p).db)));
        sqlite3_finalize(pStmt);
        return 1;
    }
    sqlite3_bind_text(pStmt, 1, to_cstring!(zDb), -1, None);
    if sqlite3_step(pStmt) == 100 && sqlite3_column_bytes(pStmt, 0) > 100 {
        memcpy(aHdr.as_mut_ptr() as *mut libc::c_void, sqlite3_column_blob(pStmt, 0), 100 as u64);
        sqlite3_finalize(pStmt);
    } else {
        eprintln!("unable to read database header");
        sqlite3_finalize(pStmt);
        return 1;
    }
    i = get2byteInt(aHdr[16..18].try_into().unwrap()) as i32;
    if i == 1 {
        i = 65536;
    }
    println!("%{:>20} {}", "database page size:", i,);
    println!("%{:>20} {}", "write format:", aHdr[18],);
    println!("%{:>20} {}", "read format:", aHdr[19],);
    println!("%{:>20} {}", "reserved bytes:", aHdr[20],);
    for field in aField {
        let ofst = field.ofst as usize;
        let val: u32 = get4byteInt(aHdr[ofst..(ofst + 4)].try_into().unwrap());
        print!("{:>20} {}", field.zName, val);
        match ofst {
            56 => {
                if val == 1 {
                    print!(" (utf8)");
                }
                if val == 2 {
                    print!(" (utf16le)");
                }
                if val == 3 {
                    print!(" (utf16be)");
                }
            }
            _ => {}
        }
        println!();
    }
    if zDb.is_empty() {
        zSchemaTab = sqlite3_mprintf(b"main.sqlite_schema\0" as *const u8 as *const i8);
    } else if zDb == "temp" {
        zSchemaTab = sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, b"sqlite_temp_schema\0" as *const u8 as *const i8);
    } else {
        zSchemaTab = sqlite3_mprintf(b"\"%w\".sqlite_schema\0" as *const u8 as *const i8, zDb);
    }

    for query in aQuery {
        let zSql: *mut i8 = sqlite3_mprintf(to_cstring!(query.zSql), zSchemaTab);
        let val_0: i32 = db_int((*p).db, zSql);
        sqlite3_free(zSql as *mut libc::c_void);
        println!("%{:>20} {}", query.zName, val_0);
    }

    sqlite3_free(zSchemaTab as *mut libc::c_void);
    sqlite3_file_control((*p).db, to_cstring!(zDb), 35, &mut iDataVersion as *mut u32 as *mut libc::c_void);
    println!("%{:>20} {}", "data version", iDataVersion,);
    return 0;
}

unsafe extern "C" fn shellDatabaseError(mut db: *mut sqlite3) -> i32 {
    let mut zErr: *const i8 = sqlite3_errmsg(db);
    eprintln!("Error: {}", cstr_to_string!(zErr));
    return 1;
}

/*
** Compare the pattern in zGlob[] against the text in z[].  Return TRUE
** if they match and FALSE (0) if they do not match.
**
** Globbing rules:
**
**      '*'       Matches any sequence of zero or more characters.
**
**      '?'       Matches exactly one character.
**
**     [...]      Matches one character from the enclosed list of
**                characters.
**
**     [^...]     Matches one character not in the enclosed list.
**
**      '#'       Matches any sequence of one or more digits with an
**                optional + or - sign in front
**
**      ' '       Any span of whitespace matches any other span of
**                whitespace.
**
** Extra whitespace at the end of z[] is ignored.
*/
fn testcase_glob(rzGlob: &str, rz: &str) -> bool {
    let mut c: u8 = 0;
    let mut c2: u8 = 0;
    let mut invert: i32 = 0;
    let mut seen: i32 = 0;

    let zGlob = rzGlob.as_bytes();
    let z = rz.as_bytes();

    let mut i = 0;
    let mut j = 0;
    loop {
        if i >= zGlob.len() {
            break;
        }
        c = zGlob[i];
        i += 1;

        if c.is_ascii_whitespace() {
            if z[j].is_ascii_whitespace() == false {
                return false;
            }
            while i < zGlob.len() && zGlob[i].is_ascii_whitespace() {
                i += 1;
            }
            while j < z.len() && z[j].is_ascii_whitespace() {
                j += 1;
            }
        } else if c == b'*' {
            loop {
                c = zGlob[i];
                i += 1;
                if !(c == b'*' || c == b'?') {
                    break;
                }
                if c == b'?' && j < z.len() {
                    return false;
                }
                j += 1;

                if i >= zGlob.len() {
                    return true;
                }
            }

            if c == b'[' {
                while j < z.len() && testcase_glob(&rzGlob[(i - 1)..], &rz[j..]) == false {
                    j += 1;
                }
                return j < z.len();
            }
            loop {
                if j >= z.len() {
                    break;
                }

                c2 = z[j];
                j += 1;

                while c2 != c {
                    if j >= z.len() {
                        return false;
                    }
                    c2 = z[j];
                    j += 1;
                }
                if testcase_glob(&rzGlob[i..], &rz[j..]) {
                    return true;
                }
            }
            return false;
        } else {
            if c == b'?' {
                if j >= z.len() {
                    return false;
                }
                j += 1;
            } else if c == b'[' {
                let mut prior_c = 0;
                seen = 0;
                invert = 0;
                if j >= z.len() {
                    return false;
                }
                c = z[j];
                j += 1;

                c2 = zGlob[i];
                i += 1;
                if c2 == b'^' {
                    invert = 1;
                    c2 = zGlob[i];
                    i += 1;
                }
                if c2 == b']' {
                    if c == b']' {
                        seen = 1;
                    }
                    c2 = zGlob[i];
                    i += 1
                }
                while i < zGlob.len() && c2 != b']' {
                    if c2 == b'-' && zGlob[i] != b']' && prior_c > 0 {
                        c2 = zGlob[i];
                        i += 1;
                        if c >= prior_c && c <= c2 {
                            seen = 1;
                        }
                        prior_c = 0;
                    } else {
                        if c == c2 {
                            seen = 1;
                        }
                        prior_c = c2;
                    }
                    c2 = zGlob[i];
                    i += 1;
                }

                if i >= zGlob.len() || seen ^ invert == 0 {
                    return false;
                }
            } else if c == b'#' {
                if (z[j] == b'-' || z[j] == b'+') && z[j + 1].is_ascii_digit() {
                    j += 1;
                }
                if z[j].is_ascii_digit() == false {
                    return false;
                }
                j += 1;
                while z[j].is_ascii_digit() {
                    j += 1;
                }
            } else {
                if c != z[j] {
                    return false;
                }
                j += 1;
            }
        }
    }
    while z[j].is_ascii_whitespace() {
        j += 1;
    }
    return j >= z.len();
}

fn optionMatch(zStr: &str, zOpt: &str) -> bool {
    if zStr.starts_with('-') == false {
        return false;
    }

    let zStr = if zStr.starts_with("--") { &zStr[2..] } else { &zStr[1..] };

    return zStr == zOpt;
}

fn shellDeleteFile(zFilename: &str) -> bool {
    match std::fs::remove_file(zFilename) {
        Ok(_) => true,
        Err(_) => false,
    }
}
fn clearTempFile(p: &mut ShellState) {
    if p.zTempFile.is_empty() {
        return;
    }
    if p.doXdgOpen != 0 {
        return;
    }
    if shellDeleteFile(&p.zTempFile) {
        return;
    }
    p.zTempFile = String::new();
}

unsafe fn newTempFile(p: *mut ShellState, zSuffix: &str) {
    clearTempFile(&mut *p);

    let mut temp: *mut i8 = std::ptr::null_mut();
    if !((*p).db).is_null() {
        sqlite3_file_control((*p).db, 0 as *const i8, 16, &mut temp as *mut *mut i8 as *mut libc::c_void);
    }

    if temp.is_null() {
        let mut r: u64 = 0;
        sqlite3_randomness(::std::mem::size_of::<u64>() as u64 as i32, &mut r as *mut u64 as *mut libc::c_void);
        let zTemp = std::env::temp_dir();
        let zTemp = zTemp.to_string_lossy();
        (*p).zTempFile = format!("{}/temp{}.{}", zTemp, r, zSuffix);
    } else {
        (*p).zTempFile = format!("{}.{}", cstr_to_string!(temp), zSuffix);
    }
}

unsafe extern "C" fn shellFkeyCollateClause(mut pCtx: *mut sqlite3_context, mut nVal: i32, mut apVal: *mut *mut sqlite3_value) {
    let mut db: *mut sqlite3 = sqlite3_context_db_handle(pCtx);
    let mut zParent: *const i8 = 0 as *const i8;
    let mut zParentCol: *const i8 = 0 as *const i8;
    let mut zParentSeq: *const i8 = 0 as *const i8;
    let mut zChild: *const i8 = 0 as *const i8;
    let mut zChildCol: *const i8 = 0 as *const i8;
    let mut zChildSeq: *const i8 = 0 as *const i8;
    let mut rc: i32 = 0;
    assert!(nVal == 4);
    zParent = sqlite3_value_text(*apVal.offset(0)) as *const i8;
    zParentCol = sqlite3_value_text(*apVal.offset(1)) as *const i8;
    zChild = sqlite3_value_text(*apVal.offset(2 as isize)) as *const i8;
    zChildCol = sqlite3_value_text(*apVal.offset(3 as isize)) as *const i8;
    sqlite3_result_text(pCtx, b"\0" as *const u8 as *const i8, -1, None);
    rc = sqlite3_table_column_metadata(
        db,
        b"main\0" as *const u8 as *const i8,
        zParent,
        zParentCol,
        0 as *mut *const i8,
        &mut zParentSeq,
        0 as *mut i32,
        0 as *mut i32,
        0 as *mut i32,
    );
    if rc == 0 {
        rc = sqlite3_table_column_metadata(
            db,
            b"main\0" as *const u8 as *const i8,
            zChild,
            zChildCol,
            0 as *mut *const i8,
            &mut zChildSeq,
            0 as *mut i32,
            0 as *mut i32,
            0 as *mut i32,
        );
    }
    if rc == 0 && sqlite3_stricmp(zParentSeq, zChildSeq) != 0 {
        let mut z: *mut i8 = sqlite3_mprintf(b" COLLATE %s\0" as *const u8 as *const i8, zParentSeq);
        sqlite3_result_text(
            pCtx,
            z,
            -1,
            ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
        );
        sqlite3_free(z as *mut libc::c_void);
    }
}

unsafe fn lintFkeyIndexes(mut db: *mut sqlite3, azArg: &Vec<String>) -> i32 {
    let mut bVerbose: i32 = 0;
    let mut bGroupByParent: i32 = 0;
    let mut zIndent: &'static str = "";
    let mut rc: i32 = 0;
    let mut pSql: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    let mut zSql: *const i8 = b"SELECT      'EXPLAIN QUERY PLAN SELECT 1 FROM ' || quote(s.name) || ' WHERE '  || group_concat(quote(s.name) || '.' || quote(f.[from]) || '=?'   || fkey_collate_clause(       f.[table], COALESCE(f.[to], p.[name]), s.name, f.[from]),' AND '),      'SEARCH ' || s.name || ' USING COVERING INDEX*('  || group_concat('*=?', ' AND ') || ')',      s.name  || '(' || group_concat(f.[from],  ', ') || ')',      f.[table] || '(' || group_concat(COALESCE(f.[to], p.[name])) || ')',      'CREATE INDEX ' || quote(s.name ||'_'|| group_concat(f.[from], '_'))  || ' ON ' || quote(s.name) || '('  || group_concat(quote(f.[from]) ||        fkey_collate_clause(          f.[table], COALESCE(f.[to], p.[name]), s.name, f.[from]), ', ')  || ');',      f.[table] FROM sqlite_schema AS s, pragma_foreign_key_list(s.name) AS f LEFT JOIN pragma_table_info AS p ON (pk-1=seq AND p.arg=f.[table]) GROUP BY s.name, f.id ORDER BY (CASE WHEN ? THEN f.[table] ELSE s.name END)\0"
        as *const u8 as *const i8;
    let mut zGlobIPK: *const i8 = b"SEARCH * USING INTEGER PRIMARY KEY (rowid=?)\0" as *const u8 as *const i8;

    for i in 2..azArg.len() {
        let n = azArg[i as usize].len();
        if n > 1 && azArg[i as usize] == "-verbose" {
            bVerbose = 1;
        } else if n > 1 && azArg[i as usize] == "-groupbyparent" {
            bGroupByParent = 1;
            zIndent = "    ";
        } else {
            eprintln!("Usage: {} {} ?-verbose? ?-groupbyparent?", azArg[0], azArg[1],);
            return 1;
        }
    }

    rc = sqlite3_create_function(
        db,
        b"fkey_collate_clause\0" as *const u8 as *const i8,
        4,
        1,
        std::ptr::null_mut(),
        Some(shellFkeyCollateClause),
        None,
        None,
    );
    if rc == 0 {
        rc = sqlite3_prepare_v2(db, zSql, -1, &mut pSql, 0 as *mut *const i8);
    }
    if rc == 0 {
        sqlite3_bind_int(pSql, 1, bGroupByParent);
    }
    if rc == 0 {
        let mut rc2: i32 = 0;
        let mut zPrev: *mut i8 = std::ptr::null_mut();
        while 100 == sqlite3_step(pSql) {
            let mut res: i32 = -1;
            let mut pExplain: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
            let mut zEQP: *const i8 = sqlite3_column_text(pSql, 0) as *const i8;
            let mut zGlob: *const i8 = sqlite3_column_text(pSql, 1) as *const i8;
            let mut zFrom: *const i8 = sqlite3_column_text(pSql, 2) as *const i8;
            let mut zTarget: *const i8 = sqlite3_column_text(pSql, 3) as *const i8;
            let mut zCI: *const i8 = sqlite3_column_text(pSql, 4) as *const i8;
            let mut zParent: *const i8 = sqlite3_column_text(pSql, 5) as *const i8;
            if zEQP.is_null() {
                continue;
            }
            if zGlob.is_null() {
                continue;
            }
            rc = sqlite3_prepare_v2(db, zEQP, -1, &mut pExplain, 0 as *mut *const i8);
            if rc != 0 {
                break;
            }
            if 100 == sqlite3_step(pExplain) {
                let mut zPlan: *const i8 = sqlite3_column_text(pExplain, 3) as *const i8;
                res = (!zPlan.is_null()
                    && (sqrite::strglob(&cstr_to_string!(zGlob), &cstr_to_string!(zPlan))
                        || sqrite::strglob(&cstr_to_string!(zGlobIPK), &cstr_to_string!(zPlan)))) as i32;
            }
            rc = sqlite3_finalize(pExplain);
            if rc != 0 {
                break;
            }
            if res < 0 {
                eprint!("Error: internal error");
                break;
            } else {
                if bGroupByParent != 0 && (bVerbose != 0 || res == 0) && (zPrev.is_null() || sqlite3_stricmp(zParent, zPrev) != 0) {
                    println!("-- Parent table {}", cstr_to_string!(zParent));
                    sqlite3_free(zPrev as *mut libc::c_void);
                    zPrev = sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, zParent);
                }
                if res == 0 {
                    println!("{}{} --> {}", zIndent, cstr_to_string!(zCI), cstr_to_string!(zTarget));
                } else if bVerbose != 0 {
                    println!(
                        "{}/* no extra indexes required for {} -> {} */",
                        zIndent,
                        cstr_to_string!(zFrom),
                        cstr_to_string!(zTarget),
                    );
                }
            }
        }
        sqlite3_free(zPrev as *mut libc::c_void);
        if rc != 0 {
            eprintln!("{}", cstr_to_string!(sqlite3_errmsg(db)));
        }
        rc2 = sqlite3_finalize(pSql);
        if rc == 0 && rc2 != 0 {
            rc = rc2;
            eprintln!("{}", cstr_to_string!(sqlite3_errmsg(db)));
        }
    } else {
        eprintln!("{}", cstr_to_string!(sqlite3_errmsg(db)));
    }
    return rc;
}

/// Implementation of ".lint" dot command.
unsafe fn lintDotCommand(pState: &mut ShellState, azArg: &Vec<String>) -> i32 {
    let n = if azArg.len() >= 2 { azArg[1].len() } else { 0 };

    if n < 1 || azArg[1] == "fkey-indexes" {
        eprintln!("Usage {} sub-command ?switches...?", azArg[0]);
        eprintln!("Where sub-commands are");
        eprintln!("    fkey-indexes");
        return 1;
    } else {
        return lintFkeyIndexes(pState.db, azArg);
    };
}

unsafe extern "C" fn shellPrepare(mut db: *mut sqlite3, mut pRc: *mut i32, mut zSql: *const i8, mut ppStmt: *mut *mut sqlite3_stmt) {
    *ppStmt = 0 as *mut sqlite3_stmt;
    if *pRc == 0 {
        let mut rc: i32 = sqlite3_prepare_v2(db, zSql, -1, ppStmt, 0 as *mut *const i8);
        if rc != 0 {
            eprintln!("sql error: {} ({})", cstr_to_string!(sqlite3_errmsg(db)), sqlite3_errcode(db));
            *pRc = rc;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn shellPreparePrintf(
    mut db: *mut sqlite3,
    mut pRc: *mut i32,
    mut ppStmt: *mut *mut sqlite3_stmt,
    mut zFmt: *const i8,
    mut args: ...
) {
    *ppStmt = 0 as *mut sqlite3_stmt;
    if *pRc == 0 {
        let mut ap: ::std::ffi::VaListImpl;
        let mut z: *mut i8 = std::ptr::null_mut();
        ap = args.clone();
        z = sqlite3_vmprintf(zFmt, ap.as_va_list());
        if z.is_null() {
            *pRc = 7 as i32;
        } else {
            shellPrepare(db, pRc, z, ppStmt);
            sqlite3_free(z as *mut libc::c_void);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn shellFinalize(mut pRc: *mut i32, mut pStmt: *mut sqlite3_stmt) {
    if !pStmt.is_null() {
        let mut db: *mut sqlite3 = sqlite3_db_handle(pStmt);
        let mut rc: i32 = sqlite3_finalize(pStmt);
        if *pRc == 0 {
            if rc != 0 {
                eprintln!("SQL error: {}", cstr_to_string!(sqlite3_errmsg(db)));
            }
            *pRc = rc;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn shellReset(mut pRc: *mut i32, mut pStmt: *mut sqlite3_stmt) {
    let mut rc: i32 = sqlite3_reset(pStmt);
    if *pRc == 0 {
        if rc != 0 {
            let mut db: *mut sqlite3 = sqlite3_db_handle(pStmt);
            eprintln!("SQL error: {}", cstr_to_string!(sqlite3_errmsg(db)));
        }
        *pRc = rc;
    }
}
unsafe extern "C" fn rc_err_oom_die(mut rc: i32) {
    if rc == 7 as i32 {
        shell_check_oom(0 as *mut libc::c_void);
    }
    assert!(rc == 0 || rc == 101);
}
const zCOL_DB: *const i8 = b":memory:\0" as *const u8 as *const i8;
unsafe fn zAutoColumn(mut zColNew: *const i8, mut pDb: *mut *mut sqlite3, mut pzRenamed: *mut *mut i8) -> *mut i8 {
    const zTabMake: *const i8 = b"CREATE TABLE ColNames( cpos INTEGER PRIMARY KEY, name TEXT, nlen INT, chop INT, reps INT, suff TEXT);CREATE VIEW RepeatedNames AS SELECT DISTINCT t.name FROM ColNames t WHERE t.name COLLATE NOCASE IN ( SELECT o.name FROM ColNames o WHERE o.cpos<>t.cpos);\0"
        as *const u8 as *const i8;
    const zTabFill: *const i8 =
        b"INSERT INTO ColNames(name,nlen,chop,reps,suff) VALUES(iif(length(?1)>0,?1,'?'),max(length(?1),1),0,0,'')\0" as *const u8
            as *const i8;
    const zHasDupes: *const i8 =
        b"SELECT count(DISTINCT (substring(name,1,nlen-chop)||suff) COLLATE NOCASE) <count(name) FROM ColNames\0" as *const u8 as *const i8;
    const zSetReps: *const i8 = b"UPDATE ColNames AS t SET reps=(SELECT count(*) FROM ColNames d  WHERE substring(t.name,1,t.nlen-t.chop)=substring(d.name,1,d.nlen-d.chop) COLLATE NOCASE)\0"
        as *const u8 as *const i8;
    const zRenameRank: *const i8 = b"WITH Lzn(nlz) AS (  SELECT 0 AS nlz  UNION  SELECT nlz+1 AS nlz FROM Lzn  WHERE EXISTS(   SELECT 1   FROM ColNames t, ColNames o   WHERE    iif(t.name IN (SELECT * FROM RepeatedNames),     printf('%s_%s',      t.name, substring(printf('%.*c%0.*d',nlz+1,'0',$1,t.cpos),2)),     t.name    )    =    iif(o.name IN (SELECT * FROM RepeatedNames),     printf('%s_%s',      o.name, substring(printf('%.*c%0.*d',nlz+1,'0',$1,o.cpos),2)),     o.name    )    COLLATE NOCASE    AND o.cpos<>t.cpos   GROUP BY t.cpos  )) UPDATE Colnames AS t SET chop = 0, suff = iif(name IN (SELECT * FROM RepeatedNames),  printf('_%s', substring(   printf('%.*c%0.*d',(SELECT max(nlz) FROM Lzn)+1,'0',1,t.cpos),2)),  '' )\0"
        as *const u8 as *const i8;
    const zCollectVar: *const i8 = b"SELECT '('||x'0a' || group_concat(  cname||' TEXT',  ','||iif((cpos-1)%4>0, ' ', x'0a'||' ')) ||')' AS ColsSpec FROM ( SELECT cpos, printf('\"%w\"',printf('%!.*s%s', nlen-chop,name,suff)) AS cname  FROM ColNames ORDER BY cpos)\0"
        as *const u8 as *const i8;
    const zRenamesDone: *const i8 = b"SELECT group_concat( printf('\"%w\" to \"%w\"',name,printf('%!.*s%s', nlen-chop, name, suff)), ','||x'0a')FROM ColNames WHERE suff<>'' OR chop!=0\0"
        as *const u8 as *const i8;
    let mut rc: i32 = 0;
    let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
    assert!(!pDb.is_null());
    if !zColNew.is_null() {
        if (*pDb).is_null() {
            if 0 != sqlite3_open(zCOL_DB, pDb) {
                return std::ptr::null_mut();
            }
            rc = sqlite3_exec(*pDb, zTabMake, None, std::ptr::null_mut(), 0 as *mut *mut i8);
            rc_err_oom_die(rc);
        }
        assert!(!(*pDb).is_null());
        rc = sqlite3_prepare_v2(*pDb, zTabFill, -1, &mut pStmt, 0 as *mut *const i8);
        rc_err_oom_die(rc);
        rc = sqlite3_bind_text(pStmt, 1, zColNew, -1, None);
        rc_err_oom_die(rc);
        rc = sqlite3_step(pStmt);
        rc_err_oom_die(rc);
        sqlite3_finalize(pStmt);
        return std::ptr::null_mut();
    } else if (*pDb).is_null() {
        return std::ptr::null_mut();
    } else {
        let mut zColsSpec: *mut i8 = std::ptr::null_mut();
        let mut hasDupes: i32 = db_int(*pDb, zHasDupes);
        if hasDupes != 0 {
            rc = sqlite3_exec(*pDb, zSetReps, None, std::ptr::null_mut(), 0 as *mut *mut i8);
            rc_err_oom_die(rc);
            rc = sqlite3_prepare_v2(*pDb, zRenameRank, -1, &mut pStmt, 0 as *mut *const i8);
            rc_err_oom_die(rc);
            sqlite3_bind_int(pStmt, 1, 2);
            rc = sqlite3_step(pStmt);
            sqlite3_finalize(pStmt);
            assert!(rc == 101);
        }
        assert!(db_int(*pDb, zHasDupes) == 0);
        rc = sqlite3_prepare_v2(*pDb, zCollectVar, -1, &mut pStmt, 0 as *mut *const i8);
        rc_err_oom_die(rc);
        rc = sqlite3_step(pStmt);
        if rc == 100 {
            zColsSpec = sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, sqlite3_column_text(pStmt, 0));
        } else {
            zColsSpec = std::ptr::null_mut();
        }
        if !pzRenamed.is_null() {
            if hasDupes == 0 {
                *pzRenamed = std::ptr::null_mut();
            } else {
                sqlite3_finalize(pStmt);
                if 0 == sqlite3_prepare_v2(*pDb, zRenamesDone, -1, &mut pStmt, 0 as *mut *const i8) && 100 == sqlite3_step(pStmt) {
                    *pzRenamed = sqlite3_mprintf(b"%s\0" as *const u8 as *const i8, sqlite3_column_text(pStmt, 0));
                } else {
                    *pzRenamed = std::ptr::null_mut();
                }
            }
        }
        sqlite3_finalize(pStmt);
        sqlite3_close(*pDb);
        *pDb = 0 as *mut sqlite3;
        return zColsSpec;
    };
}

unsafe fn do_meta_command(zLine: &str, mut p: *mut ShellState) -> i32 {
    let mut dbCols: *mut sqlite3 = 0 as *mut sqlite3;
    let mut zRenames: *mut i8 = std::ptr::null_mut();
    let mut zColDefs: *mut i8 = std::ptr::null_mut();

    if !((*p).expert.pExpert).is_null() {
        let mut errMsg = String::new();
        expertFinish(p, 1, &mut errMsg);
    }

    let razArg = shell_words::split(&zLine[1..]).unwrap();
    let mut nArg: i32 = razArg.len() as i32;

    if nArg == 0 {
        return 0;
    }

    clearTempFile(&mut *p);

    let mut rc: i32 = 0;
    'meta_command: loop {
        match razArg[0].as_str() {
            "auth" => {
                if nArg != 2 {
                    eprintln!("Usage: .auth ON|OFF");
                    rc = 1;
                    break 'meta_command;
                }

                open_db(p, 0);
                if booleanValue(&razArg[1]) != 0 {
                    sqlite3_set_authorizer((*p).db, Some(shellAuth), p as *mut libc::c_void);
                } else if (*p).bSafeModePersist != 0 {
                    sqlite3_set_authorizer((*p).db, Some(safeModeAuth), p as *mut libc::c_void);
                } else {
                    sqlite3_set_authorizer((*p).db, None, std::ptr::null_mut());
                }
            }
            "backup" | "save" => {
                let mut zDestFile = String::new();
                let mut zDb = String::new();
                let mut pDest: *mut sqlite3 = 0 as *mut sqlite3;
                let mut pBackup: *mut sqlite3_backup = 0 as *mut sqlite3_backup;
                let mut bAsync: bool = false;
                let mut zVfs: *const i8 = 0 as *const i8;
                failIfSafeMode(p, &format!("cannot run .{} in safe mode", razArg[0]));

                for z in &razArg[1..] {
                    // let z = &razArg[j as usize];
                    if z.starts_with('-') {
                        let z = if z.starts_with("--") { &z[1..] } else { z };

                        if z == "-append" {
                            zVfs = b"apndvfs\0" as *const u8 as *const i8;
                        } else if z == "-async" {
                            bAsync = true;
                        } else {
                            eprintln!("unknown option: {}", z);
                            return 1;
                        }
                    } else if zDestFile.is_empty() {
                        zDestFile = z.clone();
                    } else if zDb.is_empty() {
                        zDb = zDestFile.clone();
                        zDestFile = z.clone();
                    } else {
                        eprintln!("Usage: .backup ?DB? ?OPTIONS? FILENAME");
                        return 1;
                    }
                }
                if zDestFile.is_empty() {
                    eprintln!("missing FILENAME argument on .backup");
                    return 1;
                }
                if zDb.is_empty() {
                    zDb = "main".to_string();
                }
                rc = sqlite3_open_v2(to_cstring!(zDestFile.clone()), &mut pDest, 0x2 | 0x4 as i32, zVfs);
                if rc != 0 {
                    eprintln!("Error: cannot open \"{}\"", zDestFile.clone());
                    close_db(pDest);
                    return 1;
                }
                if bAsync {
                    sqlite3_exec(
                        pDest,
                        b"PRAGMA synchronous=OFF; PRAGMA journal_mode=OFF;\0" as *const u8 as *const i8,
                        None,
                        std::ptr::null_mut(),
                        0 as *mut *mut i8,
                    );
                }
                open_db(p, 0);
                pBackup = sqlite3_backup_init(pDest, b"main\0" as *const u8 as *const i8, (*p).db, to_cstring!(zDb));
                if pBackup.is_null() {
                    eprintln!("Error: {}", cstr_to_string!(sqlite3_errmsg(pDest)));
                    close_db(pDest);
                    return 1;
                }
                loop {
                    rc = sqlite3_backup_step(pBackup, 100);
                    if !(rc == 0) {
                        break;
                    }
                }
                sqlite3_backup_finish(pBackup);
                if rc == 101 {
                    rc = 0;
                } else {
                    eprintln!("Error: {}", cstr_to_string!(sqlite3_errmsg(pDest)));
                    rc = 1;
                }
                close_db(pDest);
            }
            "bail" => {
                if nArg != 2 {
                    eprintln!("Usage: .bail on|off");
                    rc = 1;
                    break 'meta_command;
                }

                bail_on_error = booleanValue(&razArg[1]) != 0;
            }
            "binary" => {
                if nArg != 2 {
                    eprintln!("Usage: .binary on|off");
                    rc = 1;
                    break 'meta_command;
                }

                // booleanValue(&razArg[1]) != 0;
            }
            "breakpoint" => {
                test_breakpoint();
            }
            "cd" => {
                failIfSafeMode(p, "cannot run .cd in safe mode");
                if nArg == 2 {
                    let path = std::path::Path::new(&razArg[1]);
                    let result = std::env::set_current_dir(path);
                    match result {
                        Ok(_) => rc = 0,
                        Err(_) => {
                            eprintln!("Cannot change to directory \"{}\"", razArg[1]);
                            rc = 1;
                        }
                    }
                } else {
                    eprintln!("Usage: .cd DIRECTORY");
                    rc = 1;
                }
            }
            "changes" => {
                if nArg == 2 {
                    setOrClearFlag(&mut *p, 0x20, &razArg[1]);
                } else {
                    eprintln!("Usage: .changes on|off");
                    rc = 1;
                }
            }
            "check" => {
                output_reset(p);
                if nArg != 2 {
                    eprintln!("Usage: .check GLOB-PATTERN");
                    rc = 2;
                } else {
                    let mut zRes = readFile("testcase-out.txt");
                    let zRes = std::str::from_utf8(&zRes).unwrap();
                    if zRes.is_empty() {
                        eprintln!("Error: cannot read 'testcase-out.txt");
                        rc = 2;
                    } else if testcase_glob(&razArg[1], zRes) == false {
                        eprintln!(
                            "testcase-{} FAILED\n Expected: [{}]\n      Got: [{}]",
                            ((*p).zTestcase),
                            razArg[1],
                            &zRes,
                        );
                        rc = 1;
                    } else {
                        eprintln!("testcase-{} ok", (*p).zTestcase);
                        (*p).nCheck += 1;
                    }
                }
            }
            "clone" => {
                failIfSafeMode(p, "cannot run .clone in safe mode");
                if nArg == 2 {
                    tryToClone(p, &razArg[1]);
                } else {
                    eprintln!("Usage: .clone FILENAME");
                    rc = 1;
                }
            }
            "connection" => {
                if nArg == 1 {
                    for i in 0..(*p).aAuxDb.len() {
                        let mut zFile = (*p).aAuxDb[i as usize].zDbFilename.clone();
                        if ((*p).aAuxDb[i].db).is_null() && (*p).pAuxDb != &mut *((*p).aAuxDb).as_mut_ptr().offset(i as isize) as *mut AuxDb
                        {
                            zFile = "(not open)".to_string();
                        } else if zFile.is_empty() {
                            zFile = "(memory)".to_string();
                        }

                        if (*p).pAuxDb == &mut *((*p).aAuxDb).as_mut_ptr().offset(i as isize) as *mut AuxDb {
                            println!("ACTIVE {}: {}", i, zFile);
                        } else if !((*p).aAuxDb[i].db).is_null() {
                            println!("       {}: {}", i, zFile);
                        }
                    }
                } else if nArg == 2 && razArg[1].parse::<i32>().is_ok() {
                    let mut i_0: i32 = razArg[1].parse::<i32>().unwrap();
                    if (*p).pAuxDb != &mut *((*p).aAuxDb).as_mut_ptr().offset(i_0 as isize) as *mut AuxDb && i_0 >= 0 && i_0 < 5 {
                        (*(*p).pAuxDb).db = (*p).db;
                        (*p).pAuxDb = &mut *((*p).aAuxDb).as_mut_ptr().offset(i_0 as isize) as *mut AuxDb;
                        (*p).db = (*(*p).pAuxDb).db;
                        globalDb = (*p).db;
                        (*(*p).pAuxDb).db = 0 as *mut sqlite3;
                    }
                } else if nArg == 3 && razArg[1] == "close" && razArg[2].parse::<u8>().is_ok() {
                    let mut i_1: i32 = razArg[2].parse().unwrap();
                    if !(i_1 < 0 || i_1 >= 5) {
                        if (*p).pAuxDb == &mut *((*p).aAuxDb).as_mut_ptr().offset(i_1 as isize) as *mut AuxDb {
                            eprintln!("cannot close the active database connection");
                            rc = 1;
                        } else if !((*p).aAuxDb[i_1 as usize].db).is_null() {
                            close_db((*p).aAuxDb[i_1 as usize].db);
                            (*p).aAuxDb[i_1 as usize].db = 0 as *mut sqlite3;
                        }
                    }
                } else {
                    eprintln!("Usage: .connection [close] [CONNECTION-NUMBER]");
                    rc = 1;
                }
            }
            "databases" => {
                let mut azName: Vec<_> = vec![];
                let mut nName: i32 = 0;
                let mut pStmt: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;

                open_db(p, 0);
                rc = sqlite3_prepare_v2(
                    (*p).db,
                    b"PRAGMA database_list\0" as *const u8 as *const i8,
                    -1,
                    &mut pStmt,
                    0 as *mut *const i8,
                );
                if rc != 0 {
                    eprintln!("Error: {}", cstr_to_string!(sqlite3_errmsg((*p).db)));
                    rc = 1;
                } else {
                    while sqlite3_step(pStmt) == 100 {
                        let mut zSchema = sqlite3_column_text(pStmt, 1);
                        let mut zFile_0 = sqlite3_column_text(pStmt, 2);
                        if zSchema.is_null() || zFile_0.is_null() {
                            continue;
                        }

                        azName.push(cstr_to_string!(zSchema).to_string());
                        azName.push(cstr_to_string!(zFile_0).to_string());
                        nName += 1;
                    }
                }
                sqlite3_finalize(pStmt);

                for i_2 in 0..nName as usize {
                    let eTxn: i32 = sqlite3_txn_state((*p).db, to_cstring!(azName[(i_2 * 2)].as_str()));
                    let bRdonly: i32 = sqlite3_db_readonly((*p).db, to_cstring!(azName[(i_2 * 2)].as_str()));
                    let z_0 = azName[(i_2 * 2 + 1)].as_str();

                    let temp = if !z_0.is_empty() { z_0 } else { "\"\"" };

                    println!(
                        "{}: {} {}{}",
                        &azName[(i_2 * 2)],
                        temp,
                        if bRdonly != 0 { "r/o" } else { "r/w" },
                        match eTxn {
                            0 => "",
                            1 => " read-txn",
                            _ => " write-txn",
                        }
                    );
                }
            }
            "dbconfig" => {
                const aDbConfig: [DbConfigChoices; 16] = [
                    DbConfigChoices {
                        zName: "defensive",
                        op: 1010,
                    },
                    DbConfigChoices {
                        zName: "dqs_ddl",
                        op: 1014 as i32,
                    },
                    DbConfigChoices {
                        zName: "dqs_dml",
                        op: 1013,
                    },
                    DbConfigChoices {
                        zName: "enable_fkey",
                        op: 1002,
                    },
                    DbConfigChoices {
                        zName: "enable_qpsg",
                        op: 1007,
                    },
                    DbConfigChoices {
                        zName: "enable_trigger",
                        op: 1003,
                    },
                    DbConfigChoices {
                        zName: "enable_view",
                        op: 1015,
                    },
                    DbConfigChoices {
                        zName: "fts3_tokenizer",
                        op: 1004,
                    },
                    DbConfigChoices {
                        zName: "legacy_alter_table",
                        op: 1012,
                    },
                    DbConfigChoices {
                        zName: "legacy_file_format",
                        op: 1016,
                    },
                    DbConfigChoices {
                        zName: "load_extension",
                        op: 1005,
                    },
                    DbConfigChoices {
                        zName: "no_ckpt_on_close",
                        op: 1006,
                    },
                    DbConfigChoices {
                        zName: "reset_database",
                        op: 1009,
                    },
                    DbConfigChoices {
                        zName: "trigger_eqp",
                        op: 1008,
                    },
                    DbConfigChoices {
                        zName: "trusted_schema",
                        op: 1017,
                    },
                    DbConfigChoices {
                        zName: "writable_schema",
                        op: 1011,
                    },
                ];
                let mut v: i32 = 0;
                open_db(p, 0);
                let mut found = false;
                for dbConfig in aDbConfig {
                    if nArg > 1 && razArg[1] == dbConfig.zName {
                        if nArg >= 3 {
                            sqlite3_db_config((*p).db, dbConfig.op, booleanValue(&razArg[2]), 0);
                        }
                        sqlite3_db_config((*p).db, dbConfig.op, -1, &mut v as *mut i32);
                        println!("{:>19} {}", dbConfig.zName, if v != 0 { "on" } else { "off" },);
                        if nArg > 1 {
                            break;
                        }
                        found = true
                    }
                }
                if nArg > 1 && found == false {
                    eprintln!("Error: unknown dbconfig \"{}\"", razArg[1]);
                    eprintln!("Enter \".dbconfig\" with no arguments for a list");
                }
            }
            "dbinfo" => {
                rc = shell_dbinfo_command(p, nArg, &razArg);
            }
            "dump" => {
                let mut savedShowHeader: i32 = (*p).showHeader;
                let mut savedShellFlags: i32 = (*p).shellFlgs as i32;
                (*p).shellFlgs &= !(0x8 | 0x10 | 0x40 | 0x100 | 0x200) as u32;

                let mut zLike = String::new();
                for i_3 in 1..nArg {
                    if razArg[i_3 as usize].starts_with('-') {
                        let z_1 = if razArg[i_3 as usize].starts_with("--") {
                            &razArg[i_3 as usize][2..]
                        } else {
                            &razArg[i_3 as usize][1..]
                        };

                        if z_1 == "preserve-rowids" {
                            (*p).shellFlgs |= 0x8;
                        } else if z_1 == "newlines" {
                            (*p).shellFlgs |= 0x10;
                        } else if z_1 == "data-only" {
                            (*p).shellFlgs |= 0x100;
                        } else if z_1 == "nosys" {
                            (*p).shellFlgs |= 0x200;
                        } else {
                            eprintln!("Unknown option \"{}\" on \".dump\"", &razArg[i_3 as usize]);
                            rc = 1;
                            break 'meta_command;
                        }
                    } else {
                        //FIXME: %Q is quoted, but for now just use {}
                        let zExpr = format!(
                            "name LIKE {} ESCAPE '\\' OR EXISTS (  SELECT 1 FROM sqlite_schema WHERE     name LIKE {} ESCAPE '\\' AND    sql LIKE 'CREATE VIRTUAL TABLE%%' AND    substr(o.name, 1, length(name)+1) == (name||'_'))",
                            &razArg[i_3 as usize],
                            &razArg[i_3 as usize],
                        );
                        if !zLike.is_empty() {
                            zLike = format!("{} OR {}", zLike, zExpr);
                        } else {
                            zLike = zExpr;
                        }
                    }
                }

                open_db(p, 0);
                if (*p).shellFlgs & 0x100 == 0 {
                    println!("PRAGMA foreign_keys=OFF;");
                    println!("BEGIN TRANSACTION;");
                }
                (*p).writableSchema = 0;
                (*p).showHeader = 0;
                sqlite3_exec(
                    (*p).db,
                    b"SAVEPOINT dump; PRAGMA writable_schema=ON\0" as *const u8 as *const i8,
                    None,
                    std::ptr::null_mut(),
                    0 as *mut *mut i8,
                );
                (*p).nErr = 0;
                if zLike.is_empty() {
                    zLike = format!("true");
                }
                let zSql = format!(
                    "SELECT name, type, sql FROM sqlite_schema AS o WHERE ({}) AND type=='table'  AND sql NOT NULL ORDER BY tbl_name='sqlite_sequence', rowid",
                    zLike,
                );
                run_schema_dump_query(p, &zSql);
                if (*p).shellFlgs & 0x100 == 0 {
                    let zSql = format!(
                        "SELECT sql FROM sqlite_schema AS o WHERE ({}) AND sql NOT NULL  AND type IN ('index','trigger','view')",
                        zLike,
                    );
                    run_table_dump_query(p, &zSql);
                }
                drop(zLike);
                if (*p).writableSchema != 0 {
                    println!("PRAGMA writable_schema=OFF;");
                    (*p).writableSchema = 0;
                }
                sqlite3_exec(
                    (*p).db,
                    b"PRAGMA writable_schema=OFF;\0" as *const u8 as *const i8,
                    None,
                    std::ptr::null_mut(),
                    0 as *mut *mut i8,
                );
                sqlite3_exec(
                    (*p).db,
                    b"RELEASE dump;\0" as *const u8 as *const i8,
                    None,
                    std::ptr::null_mut(),
                    0 as *mut *mut i8,
                );
                if (*p).shellFlgs & 0x100 == 0 {
                    if (*p).nErr != 0 {
                        println!("ROLLBACK; -- due to errors");
                    } else {
                        println!("COMMIT;");
                    };
                }
                (*p).showHeader = savedShowHeader;
                (*p).shellFlgs = savedShellFlags as u32;
            }
            "echo" => {
                if nArg == 2 {
                    setOrClearFlag(&mut *p, 0x40, &razArg[1]);
                } else {
                    eprintln!("Usage: .echo on|off");
                    rc = 1;
                }
            }
            "eqp" => {
                if nArg == 2 {
                    (*p).autoEQPtest = 0;
                    if (*p).autoEQPtrace != 0 {
                        if !((*p).db).is_null() {
                            sqlite3_exec(
                                (*p).db,
                                b"PRAGMA vdbe_trace=OFF;\0" as *const u8 as *const i8,
                                None,
                                std::ptr::null_mut(),
                                0 as *mut *mut i8,
                            );
                        }
                        (*p).autoEQPtrace = 0;
                    }
                    if razArg[1] == "full" {
                        (*p).autoEQP = 3;
                    } else if razArg[1] == "trigger" {
                        (*p).autoEQP = 2;
                    } else {
                        (*p).autoEQP = booleanValue(&razArg[1]) as u8;
                    }
                } else {
                    eprintln!("Usage: .eqp off|on|trace|trigger|full");
                    rc = 1;
                }
            }
            "exit" => {
                if nArg > 1 && {
                    rc = integerValue(&razArg[1]) as i32;
                    rc != 0
                } {
                    std::process::exit(rc);
                }
                rc = 2;
            }
            "explain" => {
                let mut val: i32 = 1;
                if nArg >= 2 {
                    if razArg[1] == "auto" {
                        val = 99;
                    } else {
                        val = booleanValue(&razArg[1]);
                    }
                }
                if val == 1 && (*p).mode != 9 {
                    (*p).normalMode = (*p).mode;
                    (*p).mode = 9;
                    (*p).autoExplain = 0;
                } else if val == 0 {
                    if (*p).mode == 9 {
                        (*p).mode = (*p).normalMode;
                    }
                    (*p).autoExplain = 0;
                } else if val == 99 {
                    if (*p).mode == 9 {
                        (*p).mode = (*p).normalMode;
                    }
                    (*p).autoExplain = 1;
                }
            }
            "expert" => {
                if (*p).bSafeMode != 0 {
                    eprintln!("Cannot run experimental commands such as \"{}\" in safe mode", razArg[0]);
                    rc = 1;
                } else {
                    open_db(p, 0);
                    expertDotCommand(p, &razArg);
                }
            }
            "filectrl" => {
                const aCtrl: [C2RustUnnamed_21; 9] = [
                    C2RustUnnamed_21 {
                        zCtrlName: "chunk_size",
                        ctrlCode: 6,
                        zUsage: "SIZE",
                    },
                    C2RustUnnamed_21 {
                        zCtrlName: "data_version",
                        ctrlCode: 35,
                        zUsage: "",
                    },
                    C2RustUnnamed_21 {
                        zCtrlName: "has_moved",
                        ctrlCode: 20,
                        zUsage: "",
                    },
                    C2RustUnnamed_21 {
                        zCtrlName: "lock_timeout",
                        ctrlCode: 34,
                        zUsage: "MILLISEC",
                    },
                    C2RustUnnamed_21 {
                        zCtrlName: "persist_wal",
                        ctrlCode: 10,
                        zUsage: "[BOOLEAN]",
                    },
                    C2RustUnnamed_21 {
                        zCtrlName: "psow",
                        ctrlCode: 13,
                        zUsage: "[BOOLEAN]",
                    },
                    C2RustUnnamed_21 {
                        zCtrlName: "reserve_bytes",
                        ctrlCode: 38,
                        zUsage: "[N]",
                    },
                    C2RustUnnamed_21 {
                        zCtrlName: "size_limit",
                        ctrlCode: 36,
                        zUsage: "[LIMIT]",
                    },
                    C2RustUnnamed_21 {
                        zCtrlName: "tempfilename",
                        ctrlCode: 16,
                        zUsage: "",
                    },
                ];
                let mut filectrl: i32 = -1;
                let mut iCtrl: i32 = -1;
                let mut iRes: i64 = 0;
                let mut isOk: i32 = 0;
                // let mut n2: i32 = 0;
                // let mut i_4: i32 = 0;

                open_db(p, 0);

                let mut zCmd = if nArg >= 2 { razArg[1].clone() } else { "help".to_string() };

                let mut zSchema_0 = String::new();

                let razArg = if zCmd.starts_with('-') && zCmd.starts_with("--schema") || zCmd.starts_with("-schema") && nArg >= 4 {
                    zSchema_0 = razArg[2].clone();
                    let mut args: Vec<String> = vec![];
                    args.push(razArg[0].clone());
                    for i in 3..razArg.len() {
                        args.push(razArg[i].clone());
                    }
                    nArg -= 2;
                    zCmd = args[1].clone();

                    args
                } else {
                    razArg
                };

                let zCmd = match zCmd.strip_prefix("-") {
                    Some(x) => match x.strip_prefix("-") {
                        Some(y) => y,
                        None => x,
                    },
                    None => &zCmd,
                };

                if zCmd == "help" {
                    println!("Available file-controls:");
                    for ctrl in aCtrl {
                        println!("  .filectrl {} {}", ctrl.zCtrlName, ctrl.zUsage,);
                    }
                    rc = 1;
                } else {
                    // n2 = zCmd.len() as i32;
                    for i_4 in 0..aCtrl.len() {
                        if zCmd == aCtrl[i_4].zCtrlName {
                            if filectrl < 0 {
                                filectrl = aCtrl[i_4].ctrlCode;
                                iCtrl = i_4 as i32;
                            } else {
                                eprintln!("Error: ambiguous file-control: \"{}\"\nUse \".filectrl --help\" for help", zCmd);
                                rc = 1;
                                break 'meta_command;
                            }
                        }
                    }

                    if filectrl < 0 {
                        eprintln!("Error: unknown file-control: {}\nUse \".filectrl --help\" for help", zCmd);
                    } else {
                        match filectrl {
                            36 => {
                                if !(nArg != 2 && nArg != 3) {
                                    iRes = if nArg == 3 { integerValue(&razArg[2]) } else { -1 };
                                    sqlite3_file_control((*p).db, to_cstring!(zSchema_0), 36, &mut iRes as *mut i64 as *mut libc::c_void);
                                    isOk = 1;
                                }
                            }
                            34 | 6 => {
                                if !(nArg != 3) {
                                    let mut x = integerValue(&razArg[2]) as i32;
                                    sqlite3_file_control(
                                        (*p).db,
                                        to_cstring!(zSchema_0),
                                        filectrl,
                                        &mut x as *mut i32 as *mut libc::c_void,
                                    );
                                    isOk = 2;
                                }
                            }
                            10 | 13 => {
                                if !(nArg != 2 && nArg != 3) {
                                    let mut x_0 = if nArg == 3 { booleanValue(&razArg[2]) } else { -1 };
                                    sqlite3_file_control(
                                        (*p).db,
                                        to_cstring!(zSchema_0),
                                        filectrl,
                                        &mut x_0 as *mut i32 as *mut libc::c_void,
                                    );
                                    iRes = x_0 as i64;
                                    isOk = 1;
                                }
                            }
                            35 | 20 => {
                                if !(nArg != 2) {
                                    let mut x_1: i32 = 0;
                                    sqlite3_file_control(
                                        (*p).db,
                                        to_cstring!(zSchema_0),
                                        filectrl,
                                        &mut x_1 as *mut i32 as *mut libc::c_void,
                                    );
                                    iRes = x_1 as i64;
                                    isOk = 1;
                                }
                            }
                            16 => {
                                if !(nArg != 2) {
                                    let mut z_2: *mut i8 = std::ptr::null_mut();
                                    sqlite3_file_control(
                                        (*p).db,
                                        to_cstring!(zSchema_0),
                                        filectrl,
                                        &mut z_2 as *mut *mut i8 as *mut libc::c_void,
                                    );
                                    if !z_2.is_null() {
                                        println!("{}", cstr_to_string!(z_2));
                                        sqlite3_free(z_2 as *mut libc::c_void);
                                    }
                                    isOk = 2;
                                }
                            }
                            38 => {
                                let mut x_2: i32 = 0;
                                if nArg >= 3 {
                                    x_2 = razArg[2].parse::<i32>().unwrap_or(0);
                                    sqlite3_file_control(
                                        (*p).db,
                                        to_cstring!(zSchema_0.clone()),
                                        filectrl,
                                        &mut x_2 as *mut i32 as *mut libc::c_void,
                                    );
                                }
                                x_2 = -1;
                                sqlite3_file_control(
                                    (*p).db,
                                    to_cstring!(zSchema_0.clone()),
                                    filectrl,
                                    &mut x_2 as *mut i32 as *mut libc::c_void,
                                );
                                println!("{}", x_2);
                                isOk = 2;
                            }
                            _ => {}
                        }
                    }
                }

                if isOk == 0 && iCtrl >= 0 {
                    println!("Usage: .filectrl {} {}", zCmd, aCtrl[iCtrl as usize].zUsage,);
                    rc = 1;
                } else if isOk == 1 {
                    println!("{}", iRes);
                }
            }
            "fullschema" => {
                let mut doStats: i32 = 0;
                let mut data: ShellState = (*p).clone();
                data.showHeader = 0;
                data.mode = 3;
                data.cMode = data.mode;
                if nArg == 2 && optionMatch(&razArg[1], "indent") {
                    data.mode = 11;
                    data.cMode = data.mode;
                    nArg = 1;
                }
                if nArg != 1 {
                    eprintln!("Usage: .fullschema ?--indent?");
                    rc = 1;
                    break 'meta_command;
                }

                open_db(p, 0);
                rc = sqlite3_exec(
                    (*p).db,
                    b"SELECT sql FROM  (SELECT sql sql, type type, tbl_name tbl_name, name name, rowid x     FROM sqlite_schema UNION ALL   SELECT sql, type, tbl_name, name, rowid FROM sqlite_temp_schema) WHERE type!='meta' AND sql NOTNULL AND name NOT LIKE 'sqlite_%' ORDER BY x\0"
                        as *const u8 as *const i8,
                    Some(
                        callback
                    ),
                    &mut data as *mut ShellState as *mut libc::c_void,
                    0 as *mut *mut i8,
                );

                if rc == 0 {
                    let mut pStmt_0: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                    rc = sqlite3_prepare_v2(
                        (*p).db,
                        b"SELECT rowid FROM sqlite_schema WHERE name GLOB 'sqlite_stat[134]'\0" as *const u8 as *const i8,
                        -1,
                        &mut pStmt_0,
                        0 as *mut *const i8,
                    );
                    doStats = (sqlite3_step(pStmt_0) == 100) as i32;
                    sqlite3_finalize(pStmt_0);
                }

                if doStats == 0 {
                    println!("/* No STAT tables available */");
                } else {
                    println!("ANALYZE sqlite_schema;");
                    data.mode = 5;
                    data.cMode = data.mode;
                    data.zDestTable = "sqlite_stat1".to_string();

                    let mut pzErrMsg = String::new();
                    shell_exec(&mut data, "SELECT * FROM sqlite_stat1", &mut pzErrMsg);
                    data.zDestTable = "sqlite_stat4".to_string();
                    shell_exec(&mut data, "SELECT * FROM sqlite_stat4", &mut pzErrMsg);
                    println!("ANALYZE sqlite_schema;");
                }
            }
            "headers" => {
                if nArg == 2 {
                    (*p).showHeader = booleanValue(&razArg[1]);
                    (*p).shellFlgs |= 0x80;
                } else {
                    eprintln!("Usage: .headers on|off");
                    rc = 1;
                }
            }
            "help" => {
                if nArg >= 2 {
                    let n = showHelp(&razArg[1]);
                    if n == 0 {
                        println!("Nothing matches '{}'", razArg[1]);
                    }
                } else {
                    showHelp("");
                }
            }
            "import" => {
                let mut zSchema_1 = String::new();
                let mut pStmt_1: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                let mut nCol: i32 = 0;
                let mut nByte: i32 = 0;
                let mut i_5: i32 = 0;
                let mut j_0: i32 = 0;
                let mut needCommit: i32 = 0;
                let mut nSep: i32 = 0;
                let mut zSql_0: *mut i8 = std::ptr::null_mut();
                let mut zFullTabName: *mut i8 = std::ptr::null_mut();
                let mut sCtx: ImportCtx = ImportCtx {
                    zFile: 0 as *const i8,
                    in_0: 0 as *mut FILE,
                    xCloser: None,
                    z: std::ptr::null_mut(),
                    n: 0,
                    nAlloc: 0,
                    nLine: 0,
                    nRow: 0,
                    nErr: 0,
                    bNotFirst: 0,
                    cTerm: 0,
                    cColSep: 0,
                    cRowSep: 0,
                };
                let mut xRead: Option<unsafe extern "C" fn(*mut ImportCtx) -> *mut i8> = None;
                let mut eVerbose: i32 = 0;
                let mut nSkip: i32 = 0;
                let mut useOutputMode: i32 = 1;
                let mut zCreate: *mut i8 = std::ptr::null_mut();
                failIfSafeMode(p, "cannot run .import in safe mode");
                memset(
                    &mut sCtx as *mut ImportCtx as *mut libc::c_void,
                    0,
                    ::std::mem::size_of::<ImportCtx>() as u64,
                );
                sCtx.z = sqlite3_malloc64(120) as *mut i8;
                if (sCtx.z).is_null() {
                    import_cleanup(&mut sCtx);
                    shell_out_of_memory();
                }
                if (*p).mode == 10 {
                    xRead = Some(ascii_read_one_field);
                } else {
                    xRead = Some(csv_read_one_field);
                }

                let mut zFile_1 = String::new();
                let mut zTable = String::new();

                let mut iter = razArg[1..].iter().peekable();
                while let Some(z_3) = iter.next() {
                    let z_3 = if z_3.starts_with("--") { &z_3[1..] } else { &z_3[0..] };

                    if z_3.starts_with('-') == false {
                        if zFile_1.is_empty() {
                            zFile_1 = z_3.to_string();
                        } else if zTable.is_empty() {
                            zTable = z_3.to_string();
                        } else {
                            println!("ERROR: extra argument: \"{}\".  Usage:", z_3);
                            showHelp("import");
                            rc = 1;
                            break 'meta_command;
                        }
                    } else if z_3 == "-v" {
                        eVerbose += 1;
                    } else if z_3 == "-schema" && iter.peek().is_some() {
                        let n = iter.next().unwrap();
                        zSchema_1 = n.clone();
                    } else if z_3 == "-skip" && iter.peek().is_some() {
                        let n = iter.next().unwrap();
                        nSkip = integerValue(n) as i32;
                    } else if z_3 == "-ascii" {
                        sCtx.cColSep = (*::std::mem::transmute::<&[u8; 2], &[i8; 2]>(b"\x1F\0"))[0] as i32;
                        sCtx.cRowSep = (*::std::mem::transmute::<&[u8; 2], &[i8; 2]>(b"\x1E\0"))[0] as i32;
                        xRead = Some(ascii_read_one_field);
                        useOutputMode = 0;
                    } else if z_3 == "-csv" {
                        sCtx.cColSep = ',' as i32;
                        sCtx.cRowSep = '\n' as i32;
                        xRead = Some(csv_read_one_field);
                        useOutputMode = 0;
                    } else {
                        println!("ERROR: unknown option: \"{}\".  Usage:", z_3);
                        showHelp("import");
                        rc = 1;
                        break 'meta_command;
                    }
                }

                if zTable.is_empty() {
                    println!(
                        "ERROR: missing {} argument. Usage:",
                        if zFile_1.is_empty() { "FILE" } else { "TABLE" }
                    );
                    showHelp("import");
                    rc = 1;
                    break 'meta_command;
                }

                ::std::ptr::write_volatile(&mut seenInterrupt as *mut i32, 0);
                open_db(p, 0);
                if useOutputMode != 0 {
                    nSep = (*p).colSeparator.len() as i32;
                    if nSep == 0 {
                        eprintln!("Error: non-null column separator required for import");
                        rc = 1;
                        break 'meta_command;
                    }

                    if nSep > 1 {
                        eprintln!("Error: multi-character column separators not allowed for import");
                        rc = 1;
                        break 'meta_command;
                    }

                    nSep = (*p).rowSeparator.len() as i32;
                    if nSep == 0 {
                        eprintln!("Error: non-null row separator required for import");
                        rc = 1;
                        break 'meta_command;
                    }

                    if nSep == 2 && (*p).mode == 8 && (*p).rowSeparator == "\r\n" {
                        (*p).rowSeparator = SmolStr::new("\n");
                        nSep = (*p).rowSeparator.len() as i32;
                    }
                    if nSep > 1 {
                        eprintln!("Error: multi-character row separators not allowed for import");
                        rc = 1;
                        break 'meta_command;
                    }
                    sCtx.cColSep = (*p).colSeparator.as_bytes()[0] as i32;
                    sCtx.cRowSep = (*p).rowSeparator.as_bytes()[0] as i32;
                }

                sCtx.zFile = to_cstring!(zFile_1.clone());
                sCtx.nLine = 1;
                if *(sCtx.zFile).offset(0) as i32 == '|' as i32 {
                    sCtx.in_0 = popen((sCtx.zFile).offset(1), b"r\0" as *const u8 as *const i8);
                    sCtx.zFile = b"<pipe>\0" as *const u8 as *const i8;
                    sCtx.xCloser = Some(pclose);
                } else {
                    sCtx.in_0 = fopen(sCtx.zFile, b"rb\0" as *const u8 as *const i8);
                    sCtx.xCloser = Some(fclose);
                }

                if (sCtx.in_0).is_null() {
                    eprintln!("Error: cannot open \"{}\"", zFile_1);
                    rc = 1;
                    import_cleanup(&mut sCtx);
                    break 'meta_command;
                }

                if eVerbose >= 2 || eVerbose >= 1 && useOutputMode != 0 {
                    print!("Column separator ");
                    output_c_string(&format!("{}", sCtx.cColSep as u8 as char));
                    print!(", row separator ");
                    output_c_string(&format!("{}", sCtx.cRowSep as u8 as char));
                    println!();
                }

                loop {
                    let fresh259 = nSkip;
                    nSkip -= 1;
                    if !(fresh259 > 0) {
                        break;
                    }
                    while !(xRead.expect("non-null function pointer")(&mut sCtx)).is_null() && sCtx.cTerm == sCtx.cColSep {}
                }

                if !zSchema_1.is_empty() {
                    zFullTabName = sqlite3_mprintf(b"\"%w\".\"%w\"\0" as *const u8 as *const i8, to_cstring!(zSchema_1), zTable);
                } else {
                    zFullTabName = sqlite3_mprintf(b"\"%w\"\0" as *const u8 as *const i8, to_cstring!(zTable));
                }

                zSql_0 = sqlite3_mprintf(b"SELECT * FROM %s\0" as *const u8 as *const i8, zFullTabName);
                if zSql_0.is_null() || zFullTabName.is_null() {
                    import_cleanup(&mut sCtx);
                    shell_out_of_memory();
                }

                nByte = strlen30(zSql_0);
                rc = sqlite3_prepare_v2((*p).db, zSql_0, -1, &mut pStmt_1, 0 as *mut *const i8);
                import_append_char(&mut sCtx, 0);

                let mut import_failed = false;
                'import_fail: loop {
                    if rc != 0 && sqrite::strglob("no such table: *", &cstr_to_string!(sqlite3_errmsg((*p).db))) {
                        dbCols = 0 as *mut sqlite3;
                        zRenames = std::ptr::null_mut();
                        zColDefs = std::ptr::null_mut();
                        zCreate = sqlite3_mprintf(b"CREATE TABLE %s\0" as *const u8 as *const i8, zFullTabName);
                        while !(xRead.expect("non-null function pointer")(&mut sCtx)).is_null() {
                            zAutoColumn(sCtx.z, &mut dbCols, 0 as *mut *mut i8);
                            if sCtx.cTerm != sCtx.cColSep {
                                break;
                            }
                        }
                        zColDefs = zAutoColumn(0 as *const i8, &mut dbCols, &mut zRenames);
                        if !zRenames.is_null() {
                            fprintf(
                                if stdin_is_interactive == true { stdout } else { stderr },
                                b"Columns renamed during .import %s due to duplicates:\n%s\n\0" as *const u8 as *const i8,
                                sCtx.zFile,
                                zRenames,
                            );
                            sqlite3_free(zRenames as *mut libc::c_void);
                        }
                        assert!(dbCols.is_null());
                        if zColDefs.is_null() {
                            eprintln!("{}: empty file", cstr_to_string!(sCtx.zFile));
                            import_failed = true;
                            break 'import_fail;
                        }

                        zCreate = sqlite3_mprintf(b"%z%z\n\0" as *const u8 as *const i8, zCreate, zColDefs);
                        if eVerbose >= 1 {
                            println!("{}", cstr_to_string!(zCreate));
                        }
                        rc = sqlite3_exec((*p).db, zCreate, None, std::ptr::null_mut(), 0 as *mut *mut i8);
                        if rc != 0 {
                            eprintln!("{} failed:\n{}", cstr_to_string!(zCreate), cstr_to_string!(sqlite3_errmsg((*p).db)),);
                            import_failed = true;
                            break 'import_fail;
                        }

                        sqlite3_free(zCreate as *mut libc::c_void);
                        zCreate = std::ptr::null_mut();
                        rc = sqlite3_prepare_v2((*p).db, zSql_0, -1, &mut pStmt_1, 0 as *mut *const i8);
                    }
                    if rc != 0 {
                        if !pStmt_1.is_null() {
                            sqlite3_finalize(pStmt_1);
                        }
                        eprintln!("Error: {}", cstr_to_string!(sqlite3_errmsg((*p).db)));
                        import_failed = true;
                        break 'import_fail;
                    }
                    sqlite3_free(zSql_0 as *mut libc::c_void);
                    nCol = sqlite3_column_count(pStmt_1);
                    sqlite3_finalize(pStmt_1);
                    pStmt_1 = 0 as *mut sqlite3_stmt;
                    if nCol == 0 {
                        return 0;
                    }
                    zSql_0 = sqlite3_malloc64((nByte * 2 + 20 + nCol * 2) as u64) as *mut i8;
                    if zSql_0.is_null() {
                        import_cleanup(&mut sCtx);
                        shell_out_of_memory();
                    }
                    sqlite3_snprintf(
                        nByte + 20,
                        zSql_0,
                        b"INSERT INTO %s VALUES(?\0" as *const u8 as *const i8,
                        zFullTabName,
                    );
                    j_0 = strlen30(zSql_0);
                    for _ in 1..nCol {
                        *zSql_0.offset(j_0 as isize) = ',' as i8;
                        j_0 += 1;
                        *zSql_0.offset(j_0 as isize) = '?' as i8;
                        j_0 += 1;
                    }
                    *zSql_0.offset(j_0 as isize) = ')' as i8;
                    j_0 += 1;
                    *zSql_0.offset(j_0 as isize) = 0;
                    if eVerbose >= 2 {
                        println!("Insert using: {}", cstr_to_string!(zSql_0));
                    }
                    rc = sqlite3_prepare_v2((*p).db, zSql_0, -1, &mut pStmt_1, 0 as *mut *const i8);
                    if rc != 0 {
                        eprintln!("Error: {}", cstr_to_string!(sqlite3_errmsg((*p).db)));
                        if !pStmt_1.is_null() {
                            sqlite3_finalize(pStmt_1);
                        }
                        import_failed = true;
                        break 'import_fail;
                    }

                    break;
                } // -- end of loop 'import_fail

                // import_fail:
                if import_failed {
                    sqlite3_free(zCreate as *mut libc::c_void);
                    sqlite3_free(zSql_0 as *mut libc::c_void);
                    sqlite3_free(zFullTabName as *mut libc::c_void);
                    import_cleanup(&mut sCtx);
                    rc = 1;
                    break 'meta_command;
                }

                sqlite3_free(zSql_0 as *mut libc::c_void);
                sqlite3_free(zFullTabName as *mut libc::c_void);
                needCommit = sqlite3_get_autocommit((*p).db);
                if needCommit != 0 {
                    sqlite3_exec(
                        (*p).db,
                        b"BEGIN\0" as *const u8 as *const i8,
                        None,
                        std::ptr::null_mut(),
                        0 as *mut *mut i8,
                    );
                }
                loop {
                    let mut startLine: i32 = sCtx.nLine;
                    i_5 = 0;
                    while i_5 < nCol {
                        let mut z_4: *mut i8 = xRead.expect("non-null function pointer")(&mut sCtx);
                        if z_4.is_null() && i_5 == 0 {
                            break;
                        }
                        if (*p).mode == 10 && (z_4.is_null() || *z_4.offset(0) as i32 == 0) && i_5 == 0 {
                            break;
                        }
                        sqlite3_bind_text(
                            pStmt_1,
                            i_5 + 1,
                            z_4,
                            -1,
                            ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
                        );
                        if i_5 < nCol - 1 && sCtx.cTerm != sCtx.cColSep {
                            eprintln!(
                                "{}:{}: expected {} columns but found {} - filling the rest with NULL",
                                cstr_to_string!(sCtx.zFile),
                                startLine,
                                nCol,
                                i_5 + 1,
                            );
                            i_5 += 2;
                            while i_5 <= nCol {
                                sqlite3_bind_null(pStmt_1, i_5);
                                i_5 += 1;
                            }
                        }
                        i_5 += 1;
                    }
                    if sCtx.cTerm == sCtx.cColSep {
                        loop {
                            xRead.expect("non-null function pointer")(&mut sCtx);
                            i_5 += 1;
                            if !(sCtx.cTerm == sCtx.cColSep) {
                                break;
                            }
                        }
                        eprintln!(
                            "{}:{}: expected {} columns but found {} - extras ignored",
                            cstr_to_string!(sCtx.zFile),
                            startLine,
                            nCol,
                            i_5,
                        );
                    }
                    if i_5 >= nCol {
                        sqlite3_step(pStmt_1);
                        rc = sqlite3_reset(pStmt_1);
                        if rc != 0 {
                            eprintln!(
                                "{}:{}: INSERT failed: {}",
                                cstr_to_string!(sCtx.zFile),
                                startLine,
                                cstr_to_string!(sqlite3_errmsg((*p).db)),
                            );
                            sCtx.nErr += 1;
                        } else {
                            sCtx.nRow += 1;
                        }
                    }
                    if !(sCtx.cTerm != -1) {
                        break;
                    }
                }

                import_cleanup(&mut sCtx);
                sqlite3_finalize(pStmt_1);
                if needCommit != 0 {
                    sqlite3_exec(
                        (*p).db,
                        b"COMMIT\0" as *const u8 as *const i8,
                        None,
                        std::ptr::null_mut(),
                        0 as *mut *mut i8,
                    );
                }

                if eVerbose > 0 {
                    println!(
                        "Added {} rows with {} errors using {} lines of input",
                        sCtx.nRow,
                        sCtx.nErr,
                        sCtx.nLine - 1,
                    );
                }
            }
            "imposter" => {
                let mut zSql_1: *mut i8 = std::ptr::null_mut();
                let mut zCollist: *mut i8 = std::ptr::null_mut();
                let mut pStmt_2: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                let mut tnum: i32 = 0;
                let mut isWO: i32 = 0;
                let mut lenPK: i32 = 0;
                let mut i_6: i32 = 0;
                if !(nArg == 3 || nArg == 2 && razArg[1] == "off") {
                    eprintln!("Usage: .imposter INDEX IMPOSTER\n       .imposter off");
                    rc = 1;
                    break 'meta_command;
                }

                open_db(p, 0);
                if nArg == 2 {
                    sqlite3_test_control(25, (*p).db, b"main\0" as *const u8 as *const i8, 0, 1);
                    break 'meta_command;
                }

                zSql_1 = sqlite3_mprintf(
                    b"SELECT rootpage, 0 FROM sqlite_schema WHERE name='%q' AND type='index'UNION ALL SELECT rootpage, 1 FROM sqlite_schema WHERE name='%q' AND type='table'   AND sql LIKE '%%without%%rowid%%'\0"
                        as *const u8 as *const i8,
                    to_cstring!(razArg[1].clone()),
                    to_cstring!(razArg[1].clone()),
                );
                sqlite3_prepare_v2((*p).db, zSql_1, -1, &mut pStmt_2, 0 as *mut *const i8);
                sqlite3_free(zSql_1 as *mut libc::c_void);
                if sqlite3_step(pStmt_2) == 100 {
                    tnum = sqlite3_column_int(pStmt_2, 0);
                    isWO = sqlite3_column_int(pStmt_2, 1);
                }
                sqlite3_finalize(pStmt_2);
                zSql_1 = sqlite3_mprintf(
                    b"PRAGMA index_xinfo='%q'\0" as *const u8 as *const i8,
                    to_cstring!(razArg[1].clone()),
                );
                rc = sqlite3_prepare_v2((*p).db, zSql_1, -1, &mut pStmt_2, 0 as *mut *const i8);
                sqlite3_free(zSql_1 as *mut libc::c_void);
                i_6 = 0;
                while rc == 0 && sqlite3_step(pStmt_2) == 100 {
                    let mut zLabel: [i8; 20] = [0; 20];
                    let mut zCol: *const i8 = sqlite3_column_text(pStmt_2, 2) as *const i8;
                    i_6 += 1;
                    if zCol.is_null() {
                        if sqlite3_column_int(pStmt_2, 1) == -1 {
                            zCol = b"_ROWID_\0" as *const u8 as *const i8;
                        } else {
                            sqlite3_snprintf(
                                ::std::mem::size_of::<[i8; 20]>() as u64 as i32,
                                zLabel.as_mut_ptr(),
                                b"expr%d\0" as *const u8 as *const i8,
                                i_6,
                            );
                            zCol = zLabel.as_mut_ptr();
                        }
                    }
                    if isWO != 0 && lenPK == 0 && sqlite3_column_int(pStmt_2, 5) == 0 && !zCollist.is_null() {
                        lenPK = strlen(zCollist) as i32;
                    }
                    if zCollist.is_null() {
                        zCollist = sqlite3_mprintf(b"\"%w\"\0" as *const u8 as *const i8, zCol);
                    } else {
                        zCollist = sqlite3_mprintf(b"%z,\"%w\"\0" as *const u8 as *const i8, zCollist, zCol);
                    }
                }

                sqlite3_finalize(pStmt_2);
                if i_6 == 0 || tnum == 0 {
                    eprintln!("no such index: \"{}\"", razArg[1]);
                    rc = 1;
                    sqlite3_free(zCollist as *mut libc::c_void);
                    break 'meta_command;
                }
                if lenPK == 0 {
                    lenPK = 100000;
                }
                zSql_1 = sqlite3_mprintf(
                    b"CREATE TABLE \"%w\"(%s,PRIMARY KEY(%.*s))WITHOUT ROWID\0" as *const u8 as *const i8,
                    to_cstring!(razArg[2].clone()),
                    zCollist,
                    lenPK,
                    zCollist,
                );
                sqlite3_free(zCollist as *mut libc::c_void);
                rc = sqlite3_test_control(25, (*p).db, b"main\0" as *const u8 as *const i8, 1, tnum);
                if rc == 0 {
                    rc = sqlite3_exec((*p).db, zSql_1, None, std::ptr::null_mut(), 0 as *mut *mut i8);
                    sqlite3_test_control(25, (*p).db, b"main\0" as *const u8 as *const i8, 0, 0);
                    if rc != 0 {
                        eprintln!(
                            "Error in [{}]: {}",
                            cstr_to_string!(zSql_1),
                            cstr_to_string!(sqlite3_errmsg((*p).db))
                        );
                    } else {
                        println!("{};", cstr_to_string!(zSql_1));
                        println!(
                            "WARNING: writing to an imposter table will corrupt the \"{}\" {}!",
                            razArg[1],
                            if isWO != 0 { "table" } else { "index" },
                        );
                    }
                } else {
                    eprintln!("SQLITE_TESTCTRL_IMPOSTER returns {}", rc);
                    rc = 1;
                }

                sqlite3_free(zSql_1 as *mut libc::c_void);
            }
            "limits" => {
                const aLimit: [C2RustUnnamed_20; 12] = [
                    C2RustUnnamed_20 {
                        zLimitName: "length",
                        limitCode: 0,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "sql_length",
                        limitCode: 1,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "column",
                        limitCode: 2,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "expr_depth",
                        limitCode: 3,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "compound_select",
                        limitCode: 4,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "vdbe_op",
                        limitCode: 5,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "function_arg",
                        limitCode: 6,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "attached",
                        limitCode: 7,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "like_pattern_length",
                        limitCode: 8,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "variable_number",
                        limitCode: 9,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "trigger_depth",
                        limitCode: 10,
                    },
                    C2RustUnnamed_20 {
                        zLimitName: "worker_threads",
                        limitCode: 11,
                    },
                ];

                open_db(p, 0);
                if nArg == 1 {
                    for limit in aLimit {
                        println!("{:<20} {}", limit.zLimitName, sqlite3_limit((*p).db, limit.limitCode, -1),);
                    }
                } else if nArg > 3 {
                    eprintln!("Usage: .limit NAME ?NEW-VALUE?");
                    rc = 1;
                    break 'meta_command;
                } else {
                    let mut iLimit: i32 = -1;
                    for i_7 in 0..aLimit.len() {
                        if aLimit[i_7].zLimitName.eq_ignore_ascii_case(&razArg[1]) {
                            if iLimit < 0 {
                                iLimit = i_7 as i32;
                            } else {
                                eprintln!("ambiguous limit: \"{}\"", razArg[1]);
                                rc = 1;
                                break 'meta_command;
                            }
                        }
                    }

                    if iLimit < 0 {
                        eprintln!("unknown limit: \"{}\"\nenter \".limits\" with no arguments for a list.", razArg[1],);
                        rc = 1;
                        break 'meta_command;
                    }

                    if nArg == 3 {
                        sqlite3_limit((*p).db, aLimit[iLimit as usize].limitCode, integerValue(&razArg[2]) as i32);
                    }
                    println!(
                        "{:<20} {}",
                        aLimit[iLimit as usize].zLimitName,
                        sqlite3_limit((*p).db, aLimit[iLimit as usize].limitCode, -1)
                    );
                }
            }
            "lint" => {
                open_db(p, 0);
                lintDotCommand(&mut *p, &razArg);
            }
            "load" => {
                failIfSafeMode(p, "cannot run .load in safe mode");
                if nArg < 2 {
                    eprintln!("Usage: .load FILE ?ENTRYPOINT?");
                    rc = 1;
                    break 'meta_command;
                }

                let zFile_2 = to_cstring!(razArg[1].clone());
                let zProc = if nArg >= 3 {
                    to_cstring!(razArg[2].clone())
                } else {
                    std::ptr::null_mut()
                };
                open_db(p, 0);
                let mut zErrMsg: *mut i8 = std::ptr::null_mut();
                rc = sqlite3_load_extension((*p).db, zFile_2, zProc, &mut zErrMsg);
                if rc != 0 {
                    eprintln!("Error: {}", cstr_to_string!(zErrMsg));
                    sqlite3_free(zErrMsg as *mut libc::c_void);
                    rc = 1;
                }
            }
            "log" => {
                failIfSafeMode(p, "cannot run .log in safe mode");
                if nArg != 2 {
                    eprintln!("Usage: .log FILENAME");
                    rc = 1;
                } else {
                    let mut zFile_3 = &razArg[1];
                    output_file_close((*p).pLog);
                    (*p).pLog = output_file_open(zFile_3, 0);
                }
            }
            "mode" => {
                let mut zMode = String::new();
                let mut zTabname = String::new();
                let mut cmOpts: ColModeOpts = ColModeOpts::new(60, 0, 0);

                let mut iter = razArg[1..].iter().peekable();
                while let Some(z_5) = iter.next() {
                    // let mut z_5 = &razArg[i_8 as usize];
                    if optionMatch(z_5, "wrap") && iter.peek().is_some() {
                        let n = iter.next().unwrap();
                        cmOpts.iWrap = integerValue(n) as i32;
                    } else if optionMatch(z_5, "ww") {
                        cmOpts.bWordWrap = 1;
                    } else if optionMatch(z_5, "wordwrap") && iter.peek().is_some() {
                        let n = iter.next().unwrap();
                        cmOpts.bWordWrap = booleanValue(n) as u8;
                    } else if optionMatch(z_5, "quote") {
                        cmOpts.bQuote = 1;
                    } else if optionMatch(z_5, "noquote") {
                        cmOpts.bQuote = 0;
                    } else if zMode.is_empty() {
                        if z_5 == "qbox" {
                            zMode = "box".to_string();
                            cmOpts = ColModeOpts::new(60, 1, 0);
                        } else {
                            zMode = z_5.clone();
                        }
                    } else if zTabname.is_empty() {
                        zTabname = z_5.clone();
                    } else if z_5.starts_with('-') {
                        eprintln!("unknown option: {}", z_5);
                        eprintln!("options:\n  --noquote\n  --quote\n  --wordwrap on/off\n  --wrap N\n  --ww");
                        rc = 1;
                        break 'meta_command;
                    } else {
                        eprintln!("extra argument: \"{}\"", z_5);
                        rc = 1;
                        break 'meta_command;
                    }
                }

                if zMode.is_empty() {
                    if (*p).mode == 1 || (*p).mode >= 14 && (*p).mode <= 16 {
                        println!(
                            "current output mode: {} --wrap {} --wordwrap {} --{}quote",
                            modeDescr[(*p).mode as usize],
                            (*p).cmOpts.iWrap,
                            if (*p).cmOpts.bWordWrap as i32 != 0 { "on" } else { "off" },
                            if (*p).cmOpts.bQuote as i32 != 0 { "" } else { "no" },
                        );
                    } else {
                        println!("current output mode: {}", modeDescr[(*p).mode as usize]);
                    }
                    zMode = modeDescr[(*p).mode as usize].to_string();
                }
                // n2_1 = zMode.len() as i32;
                if zMode == "lines" {
                    (*p).mode = 0;
                    (*p).rowSeparator = SmolStr::new_inline("\n");
                } else if zMode == "columns" {
                    (*p).mode = 1;
                    if (*p).shellFlgs & 0x80 == 0 {
                        (*p).showHeader = 1;
                    }
                    (*p).rowSeparator = SmolStr::new("\n");
                    (*p).cmOpts = cmOpts;
                } else if zMode == "list" {
                    (*p).mode = 2;
                    (*p).colSeparator = SmolStr::new_inline("|");
                    (*p).rowSeparator = SmolStr::new_inline("\n");
                } else if zMode == "html" {
                    (*p).mode = 4;
                } else if zMode == "tcl" {
                    (*p).mode = 7 as i32;
                    (*p).colSeparator = SmolStr::new_inline(" ");
                    (*p).rowSeparator = SmolStr::new_inline("\n");
                } else if zMode == "csv" {
                    (*p).mode = 8;
                    (*p).colSeparator = SmolStr::new_inline(",");
                    (*p).rowSeparator = SmolStr::new_inline("\r\n");
                } else if zMode == "tabs" {
                    (*p).mode = 2;
                    (*p).colSeparator = SmolStr::new_inline("\t");
                } else if zMode == "insert" {
                    (*p).mode = 5;
                    set_table_name(&mut *p, if !zTabname.is_empty() { &zTabname } else { "table" });
                } else if zMode == "quote" {
                    (*p).mode = 6;
                    (*p).colSeparator = SmolStr::new_inline(",");
                    (*p).rowSeparator = SmolStr::new_inline("\n");
                } else if zMode == "ascii" {
                    (*p).mode = 10;
                    (*p).colSeparator = SmolStr::new_inline("\x1F");
                    (*p).rowSeparator = SmolStr::new_inline("\x1E");
                } else if zMode == "markdown" {
                    (*p).mode = 14 as i32;
                    (*p).cmOpts = cmOpts;
                } else if zMode == "table" {
                    (*p).mode = 15;
                    (*p).cmOpts = cmOpts;
                } else if zMode == "box" {
                    (*p).mode = 16;
                    (*p).cmOpts = cmOpts;
                } else if zMode == "count" {
                    (*p).mode = 17;
                } else if zMode == "off" {
                    (*p).mode = 18;
                } else if zMode == "json" {
                    (*p).mode = 13;
                } else {
                    eprintln!(
                        "Error: mode should be one of: ascii box column csv html insert json line list markdown qbox quote table tabs tcl"
                    );
                    rc = 1;
                }
                (*p).cMode = (*p).mode;
            }
            "nonce" => {
                if nArg != 2 {
                    eprintln!("Usage: .nonce NONCE");
                    rc = 1;
                } else if ((*p).zNonce).is_empty() || razArg[1] == (*p).zNonce {
                    eprintln!("line {}: incorrect nonce: \"{}\"", (*p).lineno, razArg[1]);
                    std::process::exit(1);
                } else {
                    (*p).bSafeMode = 0;
                    return 0;
                }
            }
            "nullvalue" => {
                if nArg == 2 {
                    (*p).nullValue = SmolStr::new(&razArg[1]);
                } else {
                    eprintln!("Usage: .nullvalue STRING");
                    rc = 1;
                }
            }
            "open" => {
                let mut zFN = String::new();
                let mut zNewFilename = String::new();

                let mut newFlag: i32 = 0;
                let mut openMode: i32 = 0;
                for iName in 1..nArg {
                    let mut z_6 = razArg[iName as usize].clone();
                    if optionMatch(&z_6, "new") {
                        newFlag = 1;
                    } else if optionMatch(&z_6, "append") {
                        openMode = 2;
                    } else if optionMatch(&z_6, "readonly") {
                        openMode = 4;
                    } else if optionMatch(&z_6, "nofollow") {
                        (*p).openFlags |= 0x1000000;
                    } else if optionMatch(&z_6, "deserialize") {
                        openMode = 5;
                    } else if optionMatch(&z_6, "hexdb") {
                        openMode = 6;
                    } else if optionMatch(&z_6, "maxsize") && (iName + 1) < nArg {
                        (*p).szMax = integerValue(&razArg[1 + iName as usize]);
                        continue;
                    } else if z_6.starts_with('-') {
                        eprintln!("unknown option: {}", z_6);
                        rc = 1;
                        break 'meta_command;
                    } else if !zFN.is_empty() {
                        eprintln!("extra argument: \"{}\"", z_6);
                        rc = 1;
                        break 'meta_command;
                    } else {
                        zFN = z_6.clone();
                    }
                }

                close_db((*p).db);
                (*p).db = 0 as *mut sqlite3;
                (*(*p).pAuxDb).zDbFilename = String::new();

                (*p).openMode = openMode as u8;
                (*p).openFlags = 0;
                (*p).szMax = 0;
                if !zFN.is_empty() || (*p).openMode as i32 == 6 {
                    if newFlag != 0 && !zFN.is_empty() && (*p).bSafeMode == 0 {
                        shellDeleteFile(&zFN);
                    }

                    let zFN = zFN.clone();

                    if (*p).bSafeMode != 0 && (*p).openMode != 6 && zFN != "" && zFN != ":memory:" {
                        failIfSafeMode(p, "cannot open disk-based database files in safe mode\0");
                    }

                    if !zFN.is_empty() {
                        zNewFilename = zFN.clone();
                    } else {
                        zNewFilename = "".to_string();
                    }
                    (*(*p).pAuxDb).zDbFilename = zNewFilename.clone();

                    open_db(p, 0x1);
                    if ((*p).db).is_null() {
                        eprintln!("Error: cannot open '{}'", &zNewFilename);
                    } else {
                    }
                }
                if ((*p).db).is_null() {
                    // let ref mut fresh269 = (*(*p).pAuxDb).zDbFilename;
                    open_db(p, 0);
                }
            }
            "output" | "once" | "excel" => {
                // let mut bTxtMode: i32 = 0;

                let mut eMode: i32 = 0;
                // let mut bBOM: i32 = 0;
                let mut bOnce: i32 = 0;
                failIfSafeMode(p, &format!("cannot run .{} in safe mode", razArg[0]));
                if razArg[0] == "excel" {
                    eMode = 'x' as i32;
                    bOnce = 2;
                } else if razArg[0] == "once" {
                    bOnce = 1;
                }
                // let mut i_9 = 1;
                let mut zFile_4 = String::new();
                for i_9 in 1..nArg {
                    let z_7 = razArg[i_9 as usize].clone();
                    if z_7.starts_with('-') {
                        let z_7 = if z_7.starts_with("--") { &z_7[2..] } else { &z_7[1..] };

                        if z_7 == "bom" {
                            // bBOM = 1;
                        } else if razArg[0] == "excel" && z_7 == "x" {
                            eMode = 'x' as i32;
                        } else if razArg[0] == "excel" && z_7 == "e" {
                            eMode = 'e' as i32;
                        } else {
                            println!("ERROR: unknown option: \"{}\".  Usage:", razArg[i_9 as usize]);
                            showHelp(&razArg[0]);
                            rc = 1;
                            break 'meta_command;
                        }
                    } else if zFile_4.is_empty() && eMode != 'e' as i32 && eMode != 'x' as i32 {
                        zFile_4 = format!("{}", z_7);
                        if !zFile_4.is_empty() && zFile_4.starts_with('|') {
                            let mut i_9 = (i_9 as usize) + 1;
                            for i in (i_9 + 1)..nArg as usize {
                                zFile_4 = format!("{} {}", zFile_4, razArg[i]);
                            }
                            break;
                        }
                    } else {
                        println!("ERROR: extra parameter: \"{}\".  Usage:", razArg[i_9 as usize]);
                        showHelp(&razArg[0]);
                        rc = 1;
                        break 'meta_command;
                    }
                }

                if zFile_4.is_empty() {
                    zFile_4 = "stdout".to_string();
                }
                if bOnce != 0 {
                    (*p).outCount = 2;
                } else {
                    (*p).outCount = 0;
                }
                output_reset(p);
                if eMode == 'e' as i32 || eMode == 'x' as i32 {
                    (*p).doXdgOpen = 1;
                    outputModePush(&mut *p);
                    if eMode == 'x' as i32 {
                        newTempFile(p, "csv");
                        (*p).shellFlgs &= !(0x40) as u32;
                        (*p).mode = 8;
                        (*p).colSeparator = SmolStr::new_inline(",");
                        (*p).rowSeparator = SmolStr::new_inline("\r\n");
                    } else {
                        newTempFile(p, "txt");
                        // bTxtMode = 1;
                    }
                    zFile_4 = (*p).zTempFile.clone();
                }
                if zFile_4.starts_with('|') {
                    eprintln!("Error: cannot open pipe \"{}\"", &zFile_4[1..]);
                    rc = 1;
                } else {
                    if zFile_4 == "off" {
                        eprintln!("Error: cannot write to \"{}\"", zFile_4);
                    }
                    rc = 1;
                }
            }
            "parameter" => {
                // let mut current_block_1085: u64;
                open_db(p, 0);
                if nArg <= 1 {
                    showHelp("parameter");
                    break 'meta_command;
                }

                if nArg == 2 && razArg[1] == "clear" {
                    sqlite3_exec(
                        (*p).db,
                        b"DROP TABLE IF EXISTS temp.sqlite_parameters;\0" as *const u8 as *const i8,
                        None,
                        std::ptr::null_mut(),
                        0 as *mut *mut i8,
                    );
                } else if nArg == 2 && razArg[1] == "list" {
                    let mut pStmt_3: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                    let mut rx: i32 = 0;
                    let mut len: i32 = 0;
                    rx = sqlite3_prepare_v2(
                        (*p).db,
                        b"SELECT max(length(key)) FROM temp.sqlite_parameters;\0" as *const u8 as *const i8,
                        -1,
                        &mut pStmt_3,
                        0 as *mut *const i8,
                    );
                    if rx == 0 && sqlite3_step(pStmt_3) == 100 {
                        len = sqlite3_column_int(pStmt_3, 0);
                        if len > 40 {
                            len = 40;
                        }
                    }
                    sqlite3_finalize(pStmt_3);
                    pStmt_3 = 0 as *mut sqlite3_stmt;
                    if len != 0 {
                        rx = sqlite3_prepare_v2(
                            (*p).db,
                            b"SELECT key, quote(value) FROM temp.sqlite_parameters;\0" as *const u8 as *const i8,
                            -1,
                            &mut pStmt_3,
                            0 as *mut *const i8,
                        );
                        while rx == 0 && sqlite3_step(pStmt_3) == 100 {
                            println!(
                                "{content:width$} {content2}",
                                width = len as usize,
                                content = cstr_to_string!(sqlite3_column_text(pStmt_3, 0)),
                                content2 = cstr_to_string!(sqlite3_column_text(pStmt_3, 1)),
                            );
                        }
                        sqlite3_finalize(pStmt_3);
                    }
                } else if nArg == 2 && razArg[1] == "init" {
                    bind_table_init(p);
                } else if nArg == 4 && razArg[1] == "set" {
                    let mut rx_0: i32 = 0;
                    let mut zSql_2: *mut i8 = std::ptr::null_mut();
                    let mut pStmt_4: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                    let mut zKey = razArg[2].clone();
                    let mut zValue = razArg[3].clone();
                    bind_table_init(p);
                    zSql_2 = sqlite3_mprintf(
                        b"REPLACE INTO temp.sqlite_parameters(key,value)VALUES(%Q,%s);\0" as *const u8 as *const i8,
                        to_cstring!(&*zKey),
                        to_cstring!(&*zValue),
                    );
                    shell_check_oom(zSql_2 as *mut libc::c_void);
                    pStmt_4 = 0 as *mut sqlite3_stmt;
                    rx_0 = sqlite3_prepare_v2((*p).db, zSql_2, -1, &mut pStmt_4, 0 as *mut *const i8);
                    sqlite3_free(zSql_2 as *mut libc::c_void);
                    if rx_0 != 0 {
                        sqlite3_finalize(pStmt_4);
                        pStmt_4 = 0 as *mut sqlite3_stmt;
                        zSql_2 = sqlite3_mprintf(
                            b"REPLACE INTO temp.sqlite_parameters(key,value)VALUES(%Q,%Q);\0" as *const u8 as *const i8,
                            to_cstring!(&*zKey),
                            to_cstring!(&*zValue),
                        );
                        shell_check_oom(zSql_2 as *mut libc::c_void);
                        rx_0 = sqlite3_prepare_v2((*p).db, zSql_2, -1, &mut pStmt_4, 0 as *mut *const i8);
                        sqlite3_free(zSql_2 as *mut libc::c_void);
                        if rx_0 != 0 {
                            println!("Error: {}", cstr_to_string!(sqlite3_errmsg((*p).db)));
                            sqlite3_finalize(pStmt_4);
                            pStmt_4 = 0 as *mut sqlite3_stmt;
                            rc = 1;
                        }
                    }
                    sqlite3_step(pStmt_4);
                    sqlite3_finalize(pStmt_4);
                } else if nArg == 3 && razArg[1] == "unset" {
                    let mut zSql_3: *mut i8 = sqlite3_mprintf(
                        b"DELETE FROM temp.sqlite_parameters WHERE key=%Q\0" as *const u8 as *const i8,
                        to_cstring!(&*razArg[2]),
                    );
                    shell_check_oom(zSql_3 as *mut libc::c_void);
                    sqlite3_exec((*p).db, zSql_3, None, std::ptr::null_mut(), 0 as *mut *mut i8);
                    sqlite3_free(zSql_3 as *mut libc::c_void);
                } else {
                    showHelp("parameter");
                }
            }
            "print" => {
                for i_10 in 1..nArg as usize {
                    if i_10 > 1 {
                        print!(" ");
                    }
                    print!("{}", razArg[i_10]);
                }
                println!();
            }
            "progress" => {
                let mut nn: i32 = 0;
                (*p).flgProgress = 0;
                (*p).mxProgress = 0;
                (*p).nProgress = 0;

                for i_11 in 1..nArg {
                    let mut z_8 = razArg[i_11 as usize].clone();
                    if z_8.starts_with('-') == false {
                        nn = integerValue(&z_8) as i32;
                        continue;
                    }

                    let z_8 = if z_8.starts_with("--") { &z_8[2..] } else { &z_8[1..] };

                    if z_8 == "quiet" || z_8 == "q" {
                        (*p).flgProgress |= 0x1;
                    } else if z_8 == "reset" {
                        (*p).flgProgress |= 0x2 as u32;
                    } else if z_8 == "once" {
                        (*p).flgProgress |= 0x4 as i32 as u32;
                    } else if z_8 == "limit" {
                        if i_11 + 1 >= nArg {
                            eprintln!("Error: missing argument on --limit");
                            rc = 1;
                            break 'meta_command;
                        } else {
                            (*p).mxProgress = integerValue(&razArg[1 + i_11 as usize]) as i32 as u32;
                            break;
                        }
                    } else {
                        eprintln!("Error: unknown option: \"{}\"", razArg[i_11 as usize]);
                        rc = 1;
                        break 'meta_command;
                    }
                }

                open_db(p, 0);
                sqlite3_progress_handler((*p).db, nn, Some(progress_handler), p as *mut libc::c_void);
            }
            "prompt" => {
                if nArg >= 2 {
                    (*p).mainPrompt = SmolStr::new(&razArg[1]);
                }
                if nArg >= 3 {
                    (*p).continuePrompt = SmolStr::new(&razArg[2]);
                }
            }
            "quit" => {
                rc = 2;
            }
            "read" => {
                let mut savedLineno: i32 = (*p).lineno;
                failIfSafeMode(p, "cannot run .read in safe mode");
                if nArg != 2 {
                    eprintln!("Usage: .read FILE");
                    rc = 1;
                    break 'meta_command;
                }

                if razArg[1].starts_with('|') {
                    eprintln!("Error: not supported pipe");
                } else {
                    let fd = openChrSource(to_cstring!(razArg[1].clone()));
                    if fd.is_null() {
                        eprintln!("Error: cannot open \"{}\"", razArg[1]);
                        rc = 1;
                    } else {
                        rc = process_input(p);
                        fclose(fd);
                    }
                }
                (*p).lineno = savedLineno;
            }
            "restore" => {
                failIfSafeMode(p, "cannot run .restore in safe mode");
                match nArg {
                    2 | 3 => {
                        let mut zSrcFile = String::new();
                        let mut zDb_0 = String::new();
                        let mut pSrc: *mut sqlite3 = 0 as *mut sqlite3;
                        let mut pBackup_0: *mut sqlite3_backup = 0 as *mut sqlite3_backup;

                        if nArg == 2 {
                            zSrcFile = razArg[1].clone();
                            zDb_0 = "main".to_string();
                        } else if nArg == 3 {
                            zSrcFile = razArg[2].clone();
                            zDb_0 = razArg[1].clone();
                        }

                        rc = sqlite3_open(to_cstring!(zSrcFile.clone()), &mut pSrc);
                        if rc != 0 {
                            eprintln!("Error: cannot open \"{}\"", zSrcFile.clone());
                            close_db(pSrc);
                            return 1;
                        }
                        open_db(p, 0);
                        pBackup_0 = sqlite3_backup_init((*p).db, to_cstring!(zDb_0), pSrc, b"main\0" as *const u8 as *const i8);
                        if pBackup_0.is_null() {
                            eprintln!("Error: {}", cstr_to_string!(sqlite3_errmsg((*p).db)));
                            close_db(pSrc);
                            return 1;
                        }
                        let mut nTimeout: i32 = 0;
                        loop {
                            rc = sqlite3_backup_step(pBackup_0, 100);
                            if !(rc == 0 || rc == 5) {
                                break;
                            }
                            if !(rc == 5) {
                                continue;
                            }
                            let fresh277 = nTimeout;
                            nTimeout += 1;
                            if fresh277 >= 3 {
                                break;
                            }
                            sqlite3_sleep(100);
                        }
                        sqlite3_backup_finish(pBackup_0);
                        if rc == 101 {
                            rc = 0;
                        } else if rc == 5 || rc == 6 {
                            eprintln!("Error: source database is busy");
                            rc = 1;
                        } else {
                            eprintln!("Error: {}", cstr_to_string!(sqlite3_errmsg((*p).db)));
                            rc = 1;
                        }
                        close_db(pSrc);
                    }
                    _ => {
                        eprintln!("Usage: .restore ?DB? FILE");
                        rc = 1;
                    }
                }
            }
            "scanstats" => {
                if nArg == 2 {
                    (*p).scanstatsOn = booleanValue(&razArg[1]) as u8;
                    eprintln!("Warning: .scanstats not available in this build");
                } else {
                    eprintln!("Usage: .scanstats on|off");
                    rc = 1;
                }
            }
            "schema" => {
                let mut data_0: ShellState = ShellState::default();
                let mut zErrMsg_0: *mut i8 = std::ptr::null_mut();
                let mut zDiv = "(";
                let mut zName = String::new();
                let mut iSchema: i32 = 0;
                let mut bDebug: i32 = 0;
                let mut bNoSystemTabs: i32 = 0;
                // let mut ii_0: i32 = 0;
                open_db(p, 0);
                memcpy(
                    &mut data_0 as *mut ShellState as *mut libc::c_void,
                    p as *const libc::c_void,
                    ::std::mem::size_of::<ShellState>() as u64,
                );
                data_0.showHeader = 0;
                data_0.mode = 3;
                data_0.cMode = data_0.mode;

                for ii_0 in 1..nArg {
                    if optionMatch(&razArg[ii_0 as usize], "indent") {
                        data_0.mode = 11;
                        data_0.cMode = data_0.mode;
                    } else if optionMatch(&razArg[ii_0 as usize], "debug") {
                        bDebug = 1;
                    } else if optionMatch(&razArg[ii_0 as usize], "nosys") {
                        bNoSystemTabs = 1;
                    } else if razArg[ii_0 as usize].starts_with('-') {
                        eprintln!("Unknown option: \"{}\"", razArg[ii_0 as usize]);
                        rc = 1;
                        break 'meta_command;
                    } else if zName.is_empty() {
                        zName = razArg[ii_0 as usize].clone();
                    } else {
                        eprintln!("Usage: .schema ?--indent? ?--nosys? ?LIKE-PATTERN?");
                        rc = 1;
                        break 'meta_command;
                    }
                }

                let zName = zName.as_str();
                if !zName.is_empty() {
                    let isSchema = sqrite::strlike(&zName, "sqlite_master", '\\')
                        || sqrite::strlike(&zName, "sqlite_schema", '\\')
                        || sqrite::strlike(&zName, "sqlite_temp_master", '\\')
                        || sqrite::strlike(&zName, "sqlite_temp_schema", '\\');
                    if isSchema {
                        let mut new_argv: [*mut i8; 2] = [0 as *mut i8; 2];
                        let mut new_colv: [*mut i8; 2] = [0 as *mut i8; 2];
                        new_argv[0] = sqlite3_mprintf(
                            b"CREATE TABLE %s (\n  type text,\n  name text,\n  tbl_name text,\n  rootpage integer,\n  sql text\n)\0"
                                as *const u8 as *const i8,
                            zName,
                        );
                        shell_check_oom(new_argv[0] as *mut libc::c_void);
                        new_argv[1] = std::ptr::null_mut();
                        new_colv[0] = b"sql\0" as *const u8 as *const i8 as *mut i8;
                        new_colv[1] = std::ptr::null_mut();
                        callback(
                            &mut data_0 as *mut ShellState as *mut libc::c_void,
                            1,
                            new_argv.as_mut_ptr(),
                            new_colv.as_mut_ptr(),
                        );
                        sqlite3_free(new_argv[0] as *mut libc::c_void);
                    }
                }

                if zDiv.is_empty() == false {
                    let mut sSelect: ShellText = ShellText::new();
                    let mut pStmt_5: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                    rc = sqlite3_prepare_v2(
                        (*p).db,
                        b"SELECT name FROM pragma_database_list\0" as *const u8 as *const i8,
                        -1,
                        &mut pStmt_5,
                        0 as *mut *const i8,
                    );
                    if rc != 0 {
                        eprintln!("Error: {}", cstr_to_string!(sqlite3_errmsg((*p).db)));
                        sqlite3_finalize(pStmt_5);
                        rc = 1;
                        break 'meta_command;
                    }
                    sSelect.append("SELECT sql FROM", 0);
                    iSchema = 0;
                    while sqlite3_step(pStmt_5) == 100 {
                        let mut zDb_1: *const i8 = sqlite3_column_text(pStmt_5, 0) as *const i8;
                        let zDb_1 = cstr_to_string!(zDb_1);
                        let zScNum = iSchema.to_string();
                        iSchema += 1;
                        sSelect.append(zDiv, 0);
                        zDiv = " UNION ALL ";
                        sSelect.append("SELECT shell_add_schema(sql,", 0);
                        if zDb_1 != "main" {
                            sSelect.append(&zDb_1, b'\'');
                        } else {
                            sSelect.append("NULL", 0);
                        }
                        sSelect.append(",name) AS sql, type, tbl_name, name, rowid,", 0);
                        sSelect.append(&zScNum, 0);
                        sSelect.append(" AS snum, ", 0);
                        sSelect.append(&zDb_1, b'\'');
                        sSelect.append(" AS sname FROM ", 0);
                        sSelect.append(&zDb_1, quoteChar(&zDb_1));
                        sSelect.append(".sqlite_schema", 0);
                    }
                    sqlite3_finalize(pStmt_5);
                    if zName.is_empty() == false {
                        sSelect.append(
                            " UNION ALL SELECT shell_module_schema(name), 'table', name, name, name, 9e+99, 'main' FROM pragma_module_list",
                            0,
                        );
                    }
                    sSelect.append(") WHERE ", 0);
                    if !zName.is_empty() {
                        let mut zQarg: *mut i8 = sqlite3_mprintf(b"%Q\0" as *const u8 as *const i8, to_cstring!(zName));
                        let mut bGlob = false;
                        shell_check_oom(zQarg as *mut libc::c_void);
                        bGlob = zName.contains('*') || zName.contains('?') || zName.contains('[');
                        if zName.contains('.') {
                            sSelect.append("lower(printf('%s.%s',sname,tbl_name))", 0);
                        } else {
                            sSelect.append("lower(tbl_name)", 0);
                        }
                        sSelect.append(if bGlob { " GLOB " } else { " LIKE " }, 0);
                        sSelect.append(&cstr_to_string!(zQarg), 0);
                        if bGlob == false {
                            sSelect.append(" ESCAPE '\\' ", 0);
                        }
                        sSelect.append(" AND ", 0);
                        sqlite3_free(zQarg as *mut libc::c_void);
                    }
                    if bNoSystemTabs != 0 {
                        sSelect.append("name NOT LIKE 'sqlite_%%' AND ", 0);
                    }
                    sSelect.append("sql IS NOT NULL ORDER BY snum, rowid", 0);
                    if bDebug != 0 {
                        println!("SQL: {};", sSelect.as_str());
                    } else {
                        rc = sqlite3_exec(
                            (*p).db,
                            sSelect.to_cstr(),
                            Some(callback),
                            &mut data_0 as *mut ShellState as *mut libc::c_void,
                            &mut zErrMsg_0,
                        );
                    }
                    drop(sSelect);
                }

                if !zErrMsg_0.is_null() {
                    eprintln!("Error: {}", cstr_to_string!(zErrMsg_0));
                    sqlite3_free(zErrMsg_0 as *mut libc::c_void);
                    rc = 1;
                } else if rc != 0 {
                    eprintln!("Error: querying schema information");
                    rc = 1;
                } else {
                    rc = 0;
                }
            }
            "selecttrace" => {
                let mut x_3: u32 = if nArg >= 2 {
                    integerValue(&razArg[1]) as u32
                } else {
                    0xffffffff as u32
                };
                sqlite3_test_control(31, 1, &mut x_3 as *mut u32);
            }
            "selftest" => {
                let mut bIsInit: i32 = 0;
                let mut bVerbose: i32 = 0;
                let mut bSelftestExists: i32 = 0;
                // let mut i_12: i32 = 0;
                let mut k: i32 = 0;
                let mut nTest: i32 = 0;
                let mut nErr: i32 = 0;
                let mut str = ShellText::new();
                let mut pStmt_6: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                open_db(p, 0);
                for i_12 in 1..nArg {
                    let mut z_9 = razArg[i_12 as usize].as_str();
                    if z_9.starts_with("--") {
                        z_9 = &z_9[1..];
                    }
                    if z_9 == "-init" {
                        bIsInit = 1;
                    } else if z_9 == "-v" {
                        bVerbose += 1;
                    } else {
                        eprintln!("Unknown option \"{}\" on \"{}\"", razArg[i_12 as usize], razArg[0],);
                        eprintln!("Should be one of: --init -v");
                        rc = 1;
                        break 'meta_command;
                    }
                }

                if sqlite3_table_column_metadata(
                    (*p).db,
                    b"main\0" as *const u8 as *const i8,
                    b"selftest\0" as *const u8 as *const i8,
                    0 as *const i8,
                    0 as *mut *const i8,
                    0 as *mut *const i8,
                    0 as *mut i32,
                    0 as *mut i32,
                    0 as *mut i32,
                ) != 0
                {
                    bSelftestExists = 0;
                } else {
                    bSelftestExists = 1;
                }
                if bIsInit != 0 {
                    createSelftestTable(&mut *p);
                    bSelftestExists = 1;
                }
                str.append("x", 0);
                k = bSelftestExists;
                while k >= 0 {
                    if k == 1 {
                        rc = sqlite3_prepare_v2(
                            (*p).db,
                            b"SELECT tno,op,cmd,ans FROM selftest ORDER BY tno\0" as *const u8 as *const i8,
                            -1,
                            &mut pStmt_6,
                            0 as *mut *const i8,
                        );
                    } else {
                        rc = sqlite3_prepare_v2(
                            (*p).db,
                            b"VALUES(0,'memo','Missing SELFTEST table - default checks only',''),      (1,'run','PRAGMA integrity_check','ok')\0"
                                as *const u8 as *const i8,
                            -1,
                            &mut pStmt_6,
                            0 as *mut *const i8,
                        );
                    }
                    if rc != 0 {
                        eprintln!("Error querying the selftest table");
                        rc = 1;
                        sqlite3_finalize(pStmt_6);
                        break 'meta_command;
                    }
                    // i_12 = 0;
                    while sqlite3_step(pStmt_6) == 100 {
                        // i_12 += 1;

                        let mut tno: i32 = sqlite3_column_int(pStmt_6, 0);
                        let mut zOp: *const i8 = sqlite3_column_text(pStmt_6, 1) as *const i8;
                        let mut zSql_4: *const i8 = sqlite3_column_text(pStmt_6, 2) as *const i8;
                        let mut zAns: *const i8 = sqlite3_column_text(pStmt_6, 3) as *const i8;

                        if zOp.is_null() {
                            continue;
                        }

                        if zSql_4.is_null() {
                            continue;
                        }

                        if zAns.is_null() {
                            continue;
                        }

                        let zOp = cstr_to_string!(zOp);
                        let zAns = cstr_to_string!(zAns);

                        k = 0;
                        if bVerbose > 0 {
                            println!("{}: {} {}", tno, zOp, cstr_to_string!(zSql_4))
                        }
                        if zOp == "memo" {
                            println!("{}", cstr_to_string!(zSql_4));
                        } else if zOp == "run" {
                            let mut zErrMsg_1: *mut i8 = 0 as *mut i8;
                            str.clear();
                            rc = sqlite3_exec(
                                (*p).db,
                                zSql_4,
                                Some(captureOutputCallback),
                                &mut str as *mut ShellText as *mut libc::c_void,
                                &mut zErrMsg_1,
                            );
                            nTest += 1;
                            if bVerbose != 0 {
                                println!("Result: {}", &str);
                            }
                            if rc != 0 || !zErrMsg_1.is_null() {
                                nErr += 1;
                                rc = 1;
                                println!("{}: error-code-{}: {}", tno, rc, cstr_to_string!(zErrMsg_1));
                                sqlite3_free(zErrMsg_1 as *mut libc::c_void);
                            } else if zAns != str.as_str() {
                                nErr += 1;
                                rc = 1;
                                println!("{}: Expected: [{}]", tno, zAns);
                                println!("{}:      Got: [{}]", tno, str);
                            }
                        } else {
                            eprintln!("Unknown operation \"{}\" on selftest line {}", zOp, tno);
                            rc = 1;
                            break;
                        }
                    }
                    sqlite3_finalize(pStmt_6);
                    k -= 1;
                }
                println!("{} errors out of {} tests", nErr, nTest);
            }
            "separator" => {
                if nArg < 2 || nArg > 3 {
                    eprintln!("Usage: .separator COL ?ROW?");
                    rc = 1;
                }
                if nArg >= 2 {
                    (*p).colSeparator = SmolStr::new(&razArg[1]);
                }
                if nArg >= 3 {
                    (*p).rowSeparator = SmolStr::new(&razArg[2]);
                }
            }
            "sha3sum" => {
                let mut bSchema: i32 = 0;
                let mut bSeparate: i32 = 0;
                let mut iSize: i32 = 224;
                let mut bDebug_0: i32 = 0;
                let mut pStmt_7: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                let mut zSql_5: *mut i8 = std::ptr::null_mut();
                let mut zSep_0 = "";
                let mut sSql: ShellText = ShellText::new();
                let mut sQuery: ShellText = ShellText::new();
                open_db(p, 0);

                let mut zLike_0: String = String::new();

                for z in &razArg[1..] {
                    if z.starts_with('-') {
                        let z_10 = if z.starts_with("--") { &z[2..] } else { &z[1..] };

                        if z_10 == "schema" {
                            bSchema = 1;
                        } else if z_10 == "sha3-224" || z_10 == "sha3-256" || z_10 == "sha3-384" || z_10 == "sha3-512" {
                            iSize = z_10[5..].parse::<i32>().unwrap_or(0);
                        } else if z_10 == "debug" {
                            bDebug_0 = 1;
                        } else {
                            eprintln!("Unknown option \"{}\" on \"{}\"", z, razArg[0],);
                            showHelp(&razArg[0]);
                            rc = 1;
                            break 'meta_command;
                        }
                    } else if !zLike_0.is_empty() {
                        eprintln!("Usage: .sha3sum ?OPTIONS? ?LIKE-PATTERN?");
                        rc = 1;
                        break 'meta_command;
                    } else {
                        zLike_0 = z.clone();
                        bSeparate = 1;
                        if sqrite::strlike("sqlite\\_%", &z, '\\') {
                            bSchema = 1;
                        }
                    }
                }

                if bSchema != 0 {
                    zSql_5 = b"SELECT lower(name) FROM sqlite_schema WHERE type='table' AND coalesce(rootpage,0)>1 UNION ALL SELECT 'sqlite_schema' ORDER BY 1 collate nocase\0"
                as *const u8 as *const i8 as *mut i8;
                } else {
                    zSql_5 = b"SELECT lower(name) FROM sqlite_schema WHERE type='table' AND coalesce(rootpage,0)>1 AND name NOT LIKE 'sqlite_%' ORDER BY 1 collate nocase\0"
                as *const u8 as *const i8 as *mut i8;
                }
                sqlite3_prepare_v2((*p).db, zSql_5, -1, &mut pStmt_7, 0 as *mut *const i8);
                sQuery.clear();
                sSql.clear();
                sSql.append("WITH [sha3sum$query](a,b) AS(", 0);
                zSep_0 = "VALUES(";
                while 100 == sqlite3_step(pStmt_7) {
                    let mut zTab: *const i8 = sqlite3_column_text(pStmt_7, 0) as *const i8;
                    if zTab.is_null() {
                        continue;
                    }
                    if !zLike_0.is_empty() && sqrite::strlike(&zLike_0, &cstr_to_string!(zTab), '\0') == false {
                        continue;
                    }
                    let zTab = &cstr_to_string!(zTab);
                    if zTab.starts_with("sqlite_") == false {
                        sQuery.append("SELECT * FROM ", 0);
                        sQuery.append(&zTab, b'"');
                        sQuery.append(" NOT INDEXED;", 0);
                    } else if zTab == "sqlite_schema" {
                        sQuery.append("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY name;", 0);
                    } else if zTab == "sqlite_sequence" {
                        sQuery.append("SELECT name,seq FROM sqlite_sequence ORDER BY name;", 0);
                    } else if zTab == "sqlite_stat1" {
                        sQuery.append("SELECT tbl,idx,stat FROM sqlite_stat1 ORDER BY tbl,idx;", 0);
                    } else if zTab == "sqlite_stat4" {
                        sQuery.append("SELECT * FROM ", 0);
                        sQuery.append(zTab, 0);
                        sQuery.append(" ORDER BY tbl, idx, rowid;\n", 0);
                    }
                    sSql.append(zSep_0, 0);
                    sSql.append(sQuery.as_str(), b'\'');
                    sQuery.clear();
                    sSql.append(",", 0);
                    sSql.append(zTab, b'\'');
                    zSep_0 = "),(";
                }
                sqlite3_finalize(pStmt_7);
                if bSeparate != 0 {
                    zSql_5 = sqlite3_mprintf(
                        b"%s)) SELECT lower(hex(sha3_query(a,%d))) AS hash, b AS label   FROM [sha3sum$query]\0" as *const u8 as *const i8,
                        sSql.to_cstr(),
                        iSize,
                    );
                } else {
                    zSql_5 = sqlite3_mprintf(
                        b"%s)) SELECT lower(hex(sha3_query(group_concat(a,''),%d))) AS hash   FROM [sha3sum$query]\0" as *const u8
                            as *const i8,
                        sSql.to_cstr(),
                        iSize,
                    );
                }
                shell_check_oom(zSql_5 as *mut libc::c_void);
                if bDebug_0 != 0 {
                    println!("{}", cstr_to_string!(zSql_5));
                } else {
                    let mut pzErrMsg = String::new();
                    shell_exec(p, &cstr_to_string!(zSql_5), &mut pzErrMsg);
                }
                sqlite3_free(zSql_5 as *mut libc::c_void);
            }
            "shell" | "system" => {
                let mut zCmd_0: *mut i8 = std::ptr::null_mut();
                let mut i_14: i32 = 0;
                let mut x_4: i32 = 0;
                failIfSafeMode(p, &format!("cannot run .{} in safe mode", razArg[0]));
                if nArg < 2 {
                    eprintln!("Usage: .system COMMAND");
                    rc = 1;
                    break 'meta_command;
                } else {
                    zCmd_0 = sqlite3_mprintf(
                        if razArg[1].contains(' ') == false {
                            b"%s\0" as *const u8 as *const i8
                        } else {
                            b"\"%s\"\0" as *const u8 as *const i8
                        },
                        to_cstring!(razArg[1].clone()),
                    );
                    i_14 = 2;
                    while i_14 < nArg && !zCmd_0.is_null() {
                        zCmd_0 = sqlite3_mprintf(
                            if razArg[i_14 as usize].contains(' ') == false {
                                b"%z %s\0" as *const u8 as *const i8
                            } else {
                                b"%z \"%s\"\0" as *const u8 as *const i8
                            },
                            zCmd_0,
                            to_cstring!(razArg[i_14 as usize].clone()),
                        );
                        i_14 += 1;
                    }
                    x_4 = if !zCmd_0.is_null() { system(zCmd_0) } else { 1 };
                    sqlite3_free(zCmd_0 as *mut libc::c_void);
                    if x_4 != 0 {
                        eprintln!("System command returns {}", x_4);
                    }
                }
            }
            "show" => {
                const azBool: [&str; 4] = ["off", "on", "trigger", "full"];
                // let mut i_15: i32 = 0;
                if nArg != 1 {
                    eprintln!("Usage: .show");
                    rc = 1;
                    break 'meta_command;
                }

                println!("{:>12}: {}", "echo", azBool[((*p).shellFlgs & 0x40 != 0) as usize],);
                println!("{:>12}: {}", "eqp", azBool[((*p).autoEQP as i32 & 3) as usize],);
                println!(
                    "{:>12}: {}",
                    "explain",
                    if (*p).mode == 9 {
                        "on"
                    } else if (*p).autoExplain != 0 {
                        "auto"
                    } else {
                        "off"
                    }
                );
                println!("{:>12}: {}", "headers", azBool[((*p).showHeader != 0) as i32 as usize],);

                if (*p).mode == 1 || (*p).mode >= 14 && (*p).mode <= 16 {
                    println!(
                        "{:>12}: {} --wrap {} --wordwrap {} --{}quote",
                        "mode",
                        modeDescr[(*p).mode as usize],
                        (*p).cmOpts.iWrap,
                        if (*p).cmOpts.bWordWrap != 0 { "on" } else { "off" },
                        if (*p).cmOpts.bQuote != 0 { "" } else { "no" },
                    );
                } else {
                    println!("{:>12}: {}", "mode", modeDescr[(*p).mode as usize]);
                }

                print!("{:>12}: ", "nullvalue");
                output_c_string((*p).nullValue.as_str());
                println!("");

                let output = if strlen30(((*p).outfile).as_mut_ptr()) != 0 {
                    cstr_to_string!(((*p).outfile).as_mut_ptr() as *const i8).to_string()
                } else {
                    "stdout".to_string()
                };
                println!("{:>12}: {}", "output", output);
                print!("{:>12}: ", "colseparator");
                output_c_string(&(*p).colSeparator);
                println!("");

                print!("{:>12}: ", "rowseparator");
                output_c_string(&(*p).rowSeparator);
                println!("");

                let zOut = match (*p).statsOn {
                    0 => "off",
                    2 => "stmt",
                    3 => "vmstep",
                    _ => "on",
                };
                println!("{:>12}: {}", "stats", zOut);

                print!("{:>12}: ", "width");
                for i_15 in 0..(*p).nWidth as usize {
                    print!("{} ", (*p).colWidth[i_15]);
                }
                println!("");

                println!("{:>12}: {}", "filename", &(*(*p).pAuxDb).zDbFilename);
            }
            "stats" => {
                if nArg == 2 {
                    (*p).statsOn = match razArg[1].as_str() {
                        "stmt" => 2,
                        "vmstep" => 3,
                        other => booleanValue(&other) as u32,
                    }
                } else if nArg == 1 {
                    display_stats((*p).db, p, 0);
                } else {
                    eprintln!("Usage: .stats ?on|off|stmt|vmstep?");
                    rc = 1;
                }
            }
            "tables" | "indices" | "indexes" => {
                let mut pStmt_8: *mut sqlite3_stmt = 0 as *mut sqlite3_stmt;
                open_db(p, 0);
                rc = sqlite3_prepare_v2(
                    (*p).db,
                    b"PRAGMA database_list\0" as *const u8 as *const i8,
                    -1,
                    &mut pStmt_8,
                    0 as *mut *const i8,
                );
                if rc != 0 {
                    sqlite3_finalize(pStmt_8);
                    return shellDatabaseError((*p).db);
                }
                if nArg > 2 && razArg[0] != "excel" {
                    eprintln!("Usage: .indexes ?LIKE-PATTERN?");
                    rc = 1;
                    sqlite3_finalize(pStmt_8);
                    break 'meta_command;
                }

                let mut s = ShellText::new();
                while sqlite3_step(pStmt_8) == 100 {
                    let mut zDbName: *const i8 = sqlite3_column_text(pStmt_8, 1) as *const i8;
                    if !zDbName.is_null() {
                        let zDbName = cstr_to_string!(zDbName);
                        if s.len() != 0 {
                            s.append(" UNION ALL ", 0);
                        }
                        if zDbName == "main" {
                            s.append("SELECT name FROM ", 0);
                        } else {
                            s.append("SELECT ", 0);
                            s.append(&zDbName, b'\'');
                            s.append("||'.'||name FROM ", 0);
                        }
                        s.append(&zDbName, b'"');
                        s.append(".sqlite_schema ", 0);
                        if razArg[0] == "tables" {
                            s.append(
                                " WHERE type IN ('table','view')   AND name NOT LIKE 'sqlite_%'   AND name LIKE ?1",
                                0,
                            );
                        } else {
                            s.append(" WHERE type='index'   AND tbl_name LIKE ?1", 0);
                        }
                    }
                }
                rc = sqlite3_finalize(pStmt_8);
                if rc == 0 {
                    s.append(" ORDER BY 1", 0);
                    rc = sqlite3_prepare_v2((*p).db, s.to_cstr(), -1, &mut pStmt_8, 0 as *mut *const i8);
                }
                drop(s);

                if rc != 0 {
                    return shellDatabaseError((*p).db);
                }
                if nArg > 1 {
                    sqlite3_bind_text(
                        pStmt_8,
                        1,
                        to_cstring!(razArg[1].clone()),
                        -1,
                        ::std::mem::transmute::<libc::intptr_t, sqlite3_destructor_type>(-1 as libc::intptr_t),
                    );
                } else {
                    sqlite3_bind_text(pStmt_8, 1, b"%\0" as *const u8 as *const i8, -1, None);
                }
                let mut azResult = Vec::new();
                while sqlite3_step(pStmt_8) == 100 {
                    let text = cstr_to_string!(sqlite3_column_text(pStmt_8, 0));
                    azResult.push(text.to_string());
                }
                if sqlite3_finalize(pStmt_8) != 0 {
                    rc = shellDatabaseError((*p).db);
                }
                if rc == 0 && azResult.len() > 0 {
                    let maxlen = azResult.iter().map(String::len).max().unwrap_or(0);
                    let nPrintCol = std::cmp::max(80 / (maxlen + 2), 1);
                    let nPrintRow = (azResult.len() + nPrintCol - 1) / nPrintCol;
                    for i in 0..nPrintRow {
                        for j in (i..azResult.len()).step_by(nPrintRow) {
                            let zSp = if j < nPrintRow { "" } else { "  " };
                            print!("{content}{content2:width$}", content = zSp, width = maxlen, content2 = azResult[j],);
                        }
                        println!();
                    }
                }
            }
            "testcase" => {
                output_reset(p);
                eprintln!("Error: cannot open 'testcase-out.txt'");
                if nArg >= 2 {
                    (*p).zTestcase = SmolStr::new(&razArg[1]);
                } else {
                    (*p).zTestcase = SmolStr::new_inline("?");
                }
            }
            "testctrl" => {
                const aCtrl_0: [C2RustUnnamed_19; 16] = [
                    C2RustUnnamed_19 {
                        zCtrlName: "always",
                        ctrlCode: 13,
                        unSafe: 1,
                        zUsage: "BOOLEAN",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "assert",
                        ctrlCode: 12,
                        unSafe: 1,
                        zUsage: "BOOLEAN",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "byteorder",
                        ctrlCode: 22,
                        unSafe: 0,
                        zUsage: "",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "extra_schema_checks",
                        ctrlCode: 29,
                        unSafe: 0,
                        zUsage: "BOOLEAN",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "imposter",
                        ctrlCode: 25,
                        unSafe: 1,
                        zUsage: "SCHEMA ON/OFF ROOTPAGE",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "internal_functions",
                        ctrlCode: 17,
                        unSafe: 0,
                        zUsage: "",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "localtime_fault",
                        ctrlCode: 18,
                        unSafe: 0,
                        zUsage: "BOOLEAN",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "never_corrupt",
                        ctrlCode: 20,
                        unSafe: 1,
                        zUsage: "BOOLEAN",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "optimizations",
                        ctrlCode: 15,
                        unSafe: 0,
                        zUsage: "DISABLE-MASK",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "pending_byte",
                        ctrlCode: 11,
                        unSafe: 0,
                        zUsage: "OFFSET  ",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "prng_restore",
                        ctrlCode: 6,
                        unSafe: 0,
                        zUsage: "",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "prng_save",
                        ctrlCode: 5,
                        unSafe: 0,
                        zUsage: "",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "prng_seed",
                        ctrlCode: 28,
                        unSafe: 0,
                        zUsage: "SEED ?db?",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "seek_count",
                        ctrlCode: 30,
                        unSafe: 0,
                        zUsage: "",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "sorter_mmap",
                        ctrlCode: 24,
                        unSafe: 0,
                        zUsage: "NMAX",
                    },
                    C2RustUnnamed_19 {
                        zCtrlName: "tune",
                        ctrlCode: 32,
                        unSafe: 1,
                        zUsage: "ID VALUE",
                    },
                ];

                let mut rc2: i32 = 0;
                let mut isOk_0: i32 = 0;
                // let mut i_17: i32 = 0;
                // let mut n2_3: i32 = 0;
                open_db(p, 0);
                let mut zCmd_1 = if nArg >= 2 { &razArg[1] } else { "help" };
                if zCmd_1.starts_with('-') {
                    zCmd_1 = &zCmd_1[1..];
                    if zCmd_1.starts_with('-') {
                        zCmd_1 = &zCmd_1[1..];
                    }
                }
                if zCmd_1 == "help" {
                    println!("Available test-controls:");

                    for ctrl in aCtrl_0 {
                        println!("  .testctrl {} {}", ctrl.zCtrlName, ctrl.zUsage,);
                    }
                    rc = 1;
                    break 'meta_command;
                }

                let mut testctrl: i32 = -1;
                let mut iCtrl_0: i32 = -1;
                for (i, ctrl) in aCtrl_0.iter().enumerate() {
                    if zCmd_1 == ctrl.zCtrlName {
                        if testctrl < 0 {
                            testctrl = ctrl.ctrlCode;
                            iCtrl_0 = i as i32;
                        } else {
                            eprintln!("Error: ambiguous test-control: \"{}\"\nUse \".testctrl --help\" for help", zCmd_1);
                            rc = 1;
                            break 'meta_command;
                        }
                    }
                }

                if testctrl < 0 {
                    eprintln!("Error: unknown test-control: {}\nUse \".testctrl --help\" for help", zCmd_1);
                } else if aCtrl_0[iCtrl_0 as usize].unSafe != 0 && (*p).bSafeMode as i32 != 0 {
                    eprintln!(
                        "line {}: \".testctrl {}\" may not be used in safe mode",
                        (*p).lineno,
                        aCtrl_0[iCtrl_0 as usize].zCtrlName,
                    );
                    std::process::exit(1);
                } else {
                    match testctrl {
                        15 => {
                            if nArg == 3 {
                                let opt: u32 = razArg[2].parse::<u32>().unwrap_or(0);
                                rc2 = sqlite3_test_control(testctrl, (*p).db, opt);
                                isOk_0 = 3;
                            }
                        }
                        5 | 6 | 22 => {
                            if nArg == 2 {
                                rc2 = sqlite3_test_control(testctrl);
                                isOk_0 = if testctrl == 22 { 1 } else { 3 };
                            }
                        }
                        11 => {
                            if nArg == 3 {
                                let opt_0: u32 = integerValue(&razArg[2]) as u32;
                                rc2 = sqlite3_test_control(testctrl, opt_0);
                                isOk_0 = 3;
                            }
                        }
                        28 => {
                            if nArg == 3 || nArg == 4 {
                                let mut ii_2: i32 = integerValue(&razArg[2]) as i32;
                                let mut db: *mut sqlite3 = 0 as *mut sqlite3;
                                if ii_2 == 0 && razArg[2] == "random" {
                                    sqlite3_randomness(
                                        ::std::mem::size_of::<i32>() as u64 as i32,
                                        &mut ii_2 as *mut i32 as *mut libc::c_void,
                                    );
                                    println!("-- random seed: {}", ii_2);
                                }
                                if nArg == 3 {
                                    db = 0 as *mut sqlite3;
                                } else {
                                    db = (*p).db;
                                    sqlite3_table_column_metadata(
                                        db,
                                        0 as *const i8,
                                        b"x\0" as *const u8 as *const i8,
                                        0 as *const i8,
                                        0 as *mut *const i8,
                                        0 as *mut *const i8,
                                        0 as *mut i32,
                                        0 as *mut i32,
                                        0 as *mut i32,
                                    );
                                }
                                rc2 = sqlite3_test_control(testctrl, ii_2, db);
                                isOk_0 = 3;
                            }
                        }
                        12 | 13 => {
                            if nArg == 3 {
                                let mut opt_1: i32 = booleanValue(&razArg[2]);
                                rc2 = sqlite3_test_control(testctrl, opt_1);
                                isOk_0 = 1;
                            }
                        }
                        18 | 20 => {
                            if nArg == 3 {
                                let opt_2: i32 = booleanValue(&razArg[2]);
                                rc2 = sqlite3_test_control(testctrl, opt_2);
                                isOk_0 = 3;
                            }
                        }
                        17 => {
                            rc2 = sqlite3_test_control(testctrl, (*p).db);
                            isOk_0 = 3;
                        }
                        25 => {
                            if nArg == 5 {
                                rc2 = sqlite3_test_control(
                                    testctrl,
                                    (*p).db,
                                    to_cstring!(razArg[2].clone()),
                                    integerValue(&razArg[3]),
                                    integerValue(&razArg[4]),
                                );
                                isOk_0 = 3;
                            }
                        }
                        30 => {
                            let mut x_5: u64 = 0;
                            rc2 = sqlite3_test_control(testctrl, (*p).db, &mut x_5 as *mut u64);
                            println!("{}", x_5);
                            isOk_0 = 3;
                        }
                        24 => {
                            if nArg == 3 {
                                let opt_3: i32 = integerValue(&razArg[2]) as i32;
                                rc2 = sqlite3_test_control(testctrl, (*p).db, opt_3);
                                isOk_0 = 3;
                            }
                        }
                        _ => {}
                    }
                }
                if isOk_0 == 0 && iCtrl_0 >= 0 {
                    println!("Usage: .testctrl {} {}", zCmd_1, aCtrl_0[iCtrl_0 as usize].zUsage,);
                    rc = 1;
                } else if isOk_0 == 1 {
                    print!("{}", rc2);
                } else if isOk_0 == 2 {
                    print!("{:x}", rc2);
                }
            }
            "timeout" => {
                open_db(p, 0);
                sqlite3_busy_timeout((*p).db, if nArg >= 2 { integerValue(&razArg[1]) as i32 } else { 0 });
            }
            "timer" => {
                if nArg == 2 {
                    enableTimer = booleanValue(&razArg[1]);
                    if enableTimer != 0 && 1 == 0 {
                        eprintln!("Error: timer not available on this system");
                        enableTimer = 0;
                    }
                } else {
                    eprintln!("Usage: .timer on|off");
                    rc = 1;
                }
            }
            "trace" => {
                let mut mType: i32 = 0;
                open_db(p, 0);
                for jj in 1..nArg {
                    let mut z_11 = razArg[jj as usize].clone();
                    if z_11.starts_with('-') {
                        if optionMatch(&z_11, "expanded") {
                            (*p).eTraceType = 1;
                        } else if optionMatch(&z_11, "plain") {
                            (*p).eTraceType = 0;
                        } else if optionMatch(&z_11, "profile") {
                            mType |= 0x2;
                        } else if optionMatch(&z_11, "row") {
                            mType |= 0x4 as i32;
                        } else if optionMatch(&z_11, "stmt") {
                            mType |= 0x1;
                        } else if optionMatch(&z_11, "close") {
                            mType |= 0x8;
                        } else {
                            eprintln!("Unknown option \"{}\" on \".trace\"", z_11);
                            rc = 1;
                            break 'meta_command;
                        }
                    } else {
                        output_file_close((*p).traceOut);
                        (*p).traceOut = output_file_open(&razArg[1], 0);
                    }
                }

                if ((*p).traceOut).is_null() {
                    sqlite3_trace_v2((*p).db, 0, None, std::ptr::null_mut());
                } else {
                    if mType == 0 {
                        mType = 0x1;
                    }
                    sqlite3_trace_v2((*p).db, mType as u32, Some(sql_trace_callback), p as *mut libc::c_void);
                }
            }
            "version" => {
                println!(
                    "SQLite {} {}",
                    cstr_to_string!(sqlite3_libversion()),
                    cstr_to_string!(sqlite3_sourceid()),
                );
                println!("clang-10.0.0");
            }
            "vfsinfo" => {
                let mut zDbName_0 = if nArg == 2 { &razArg[1] } else { "main" };
                let mut pVfs: *mut sqlite3_vfs = std::ptr::null_mut();
                if !((*p).db).is_null() {
                    sqlite3_file_control(
                        (*p).db,
                        to_cstring!(zDbName_0),
                        27 as i32,
                        &mut pVfs as *mut *mut sqlite3_vfs as *mut libc::c_void,
                    );
                    if !pVfs.is_null() {
                        println!("vfs.zName      = \"{}\"", cstr_to_string!((*pVfs).zName));
                        println!("vfs.iVersion   = {}", (*pVfs).iVersion);
                        println!("vfs.szOsFile   = {}", (*pVfs).szOsFile);
                        println!("vfs.mxPathname = {}", (*pVfs).mxPathname);
                    }
                }
            }
            "vfslist" => {
                let mut pVfs_0: *mut sqlite3_vfs = std::ptr::null_mut();
                let mut pCurrent: *mut sqlite3_vfs = std::ptr::null_mut();
                if !((*p).db).is_null() {
                    sqlite3_file_control(
                        (*p).db,
                        b"main\0" as *const u8 as *const i8,
                        27 as i32,
                        &mut pCurrent as *mut *mut sqlite3_vfs as *mut libc::c_void,
                    );
                }
                pVfs_0 = sqlite3_vfs_find(0 as *const i8);
                while !pVfs_0.is_null() {
                    println!(
                        "vfs.zName      = \"{}\"{}",
                        cstr_to_string!((*pVfs_0).zName),
                        if pVfs_0 == pCurrent { "  <--- CURRENT" } else { "" },
                    );
                    println!("vfs.iVersion   = {}", (*pVfs_0).iVersion);
                    println!("vfs.szOsFile   = {}", (*pVfs_0).szOsFile);
                    println!("vfs.mxPathname = {}", (*pVfs_0).mxPathname);
                    if !((*pVfs_0).pNext).is_null() {
                        println!("-----------------------------------");
                    }
                    pVfs_0 = (*pVfs_0).pNext;
                }
            }
            "vfsname" => {
                let zDbName_1 = if nArg == 2 { &razArg[1] } else { "main" };
                let mut zVfsName: *mut i8 = std::ptr::null_mut();
                if !((*p).db).is_null() {
                    sqlite3_file_control(
                        (*p).db,
                        to_cstring!(zDbName_1),
                        12,
                        &mut zVfsName as *mut *mut i8 as *mut libc::c_void,
                    );
                    if !zVfsName.is_null() {
                        println!("{}", cstr_to_string!(zVfsName));
                        sqlite3_free(zVfsName as *mut libc::c_void);
                    }
                }
            }
            "wheretrace" => {
                let mut x_6: u32 = if nArg >= 2 {
                    integerValue(&razArg[1]) as u32
                } else {
                    0xffffffff as u32
                };
                sqlite3_test_control(31, 3, &mut x_6 as *mut u32);
            }
            "width" => {
                assert!(nArg <= 52);
                (*p).nWidth = nArg - 1;
                (*p).colWidth.resize((((*p).nWidth + 1) * 2) as usize, 0);

                for j in 1..nArg as usize {
                    (*p).colWidth[j - 1] = integerValue(&razArg[j]) as i32;
                }
                if (*p).nWidth != 0 {
                    (*p).actualWidth = (*p).colWidth[(*p).nWidth as usize..].to_vec();
                }
            }
            _ => {
                eprintln!(
                    "Error: unknown command or invalid arguments:  \"{}\". Enter \".help\" for help",
                    razArg[0]
                );
                rc = 1;
            }
        };
        break;
    }

    // meta_command_exit:
    if (*p).outCount != 0 {
        (*p).outCount -= 1;
        if (*p).outCount == 0 {
            output_reset(p);
        }
    }
    (*p).bSafeMode = (*p).bSafeModePersist;
    return rc;
}

/*
** Scan line for classification to guide shell's handling.
** The scan is resumable for subsequent lines when prior
** return values are passed as the 2nd argument.
*/
fn quickscan(zLine: &str, mut qss: QuickScanState) -> QuickScanState {
    let mut current_block: u64;
    let mut cin: u8 = 0;
    let mut cWait: u8 = qss as u8;

    let mut zLine = zLine.as_bytes();

    if cWait == 0 {
        current_block = 16016083347208075581;
    } else {
        current_block = 16203760046146113240;
    }
    'c_88596: loop {
        match current_block {
            16016083347208075581 => {
                assert!(cWait as i32 == 0);
                's_19: loop {
                    if zLine.len() != 0 {
                        break 'c_88596;
                    }
                    cin = zLine[0];
                    zLine = &zLine[1..];
                    if (cin as char).is_ascii_whitespace() {
                        continue;
                    }
                    match cin {
                        45 => {
                            if zLine.starts_with(&[b'-']) {
                                current_block = 1109700713171191020;
                            } else {
                                loop {
                                    if !(zLine.len() != 0) {
                                        break;
                                    }

                                    zLine = &zLine[1..];
                                    cin = zLine[0];

                                    if cin == b'\n' {
                                        current_block = 16016083347208075581;
                                        break 's_19;
                                    }
                                }
                                return qss;
                            }
                        }
                        59 => {
                            qss = (qss as u32 | QSS_EndingSemi as u32) as QuickScanState;
                            continue;
                        }
                        47 => {
                            if zLine.starts_with(&[b'*']) {
                                zLine = &zLine[1..];
                                cWait = b'*';
                                qss = (cWait as u32 | qss as u32 & QSS_ScanMask as i32 as u32) as QuickScanState;
                                current_block = 16203760046146113240;
                                break;
                            } else {
                                current_block = 1109700713171191020;
                            }
                        }
                        91 => {
                            cin = b']';
                            current_block = 14767508466678152934;
                        }
                        96 | 39 | 34 => {
                            current_block = 14767508466678152934;
                        }
                        _ => {
                            current_block = 1109700713171191020;
                        }
                    }
                    match current_block {
                        1109700713171191020 => {
                            qss = (qss as u32 & !(QSS_EndingSemi as i32) as u32 | QSS_HasDark as i32 as u32) as QuickScanState;
                        }
                        _ => {
                            cWait = cin;
                            qss = (QSS_HasDark as i32 | cWait as i32) as QuickScanState;
                            current_block = 16203760046146113240;
                            break;
                        }
                    }
                }
            }
            _ => {
                if zLine.len() == 0 {
                    break;
                }

                cin = zLine[0];
                zLine = &zLine[1..];

                if !(cin == cWait) {
                    current_block = 16203760046146113240;
                    continue;
                }
                match cWait as i32 {
                    42 => {
                        if zLine.starts_with(&[b'/']) {
                            current_block = 16203760046146113240;
                            continue;
                        }
                        zLine = &zLine[1..];
                        cWait = 0;
                        qss = (0 | qss as u32 & QSS_ScanMask as i32 as u32) as QuickScanState;
                        current_block = 16016083347208075581;
                        continue;
                    }
                    96 | 39 | 34 => {
                        if zLine.starts_with(&[cWait]) {
                            zLine = &zLine[1..];
                            current_block = 16203760046146113240;
                            continue;
                        }
                    }
                    93 => {}
                    _ => {
                        unreachable!();
                        continue;
                    }
                }
                cWait = 0;
                qss = (0 | qss as u32 & QSS_ScanMask as i32 as u32) as QuickScanState;
                current_block = 16016083347208075581;
            }
        }
    }
    return qss;
}

/*
** Return TRUE if the line typed in is an SQL command terminator other
** than a semi-colon.  The SQL Server style "go" command is understood
** as is the Oracle "/".
*/
fn line_is_command_terminator(zLine: &str) -> bool {
    let zLine = zLine.trim_start();
    let zLine = if zLine.starts_with('/') {
        &zLine[1..]
    } else if zLine[0..2].eq_ignore_ascii_case("go") {
        &zLine[2..]
    } else {
        return false;
    };

    return quickscan(zLine, QSS_Start) == QSS_Start;
}

/// Return true if zSql is a complete SQL statement.  Return false if it
/// ends in the middle of a string literal or C-style comment.
fn line_is_complete(zSql: &str) -> bool {
    if zSql.is_empty() {
        return true;
    }

    let zSql = format!("{zSql};");
    sqrite::is_complete(&zSql)
}

/// Run a single line of SQL.  Return the number of errors.
unsafe fn runOneSqlLine(mut p: *mut ShellState, zSql: &str, startline: i32) -> i32 {
    let mut rc: i32 = 0;

    open_db(p, 0);
    let zSql = if (*p).shellFlgs & 0x4 != 0 {
        std::borrow::Cow::from(resolve_backslashes(zSql))
    } else {
        std::borrow::Cow::from(zSql)
    };

    if (*p).flgProgress & 0x2 != 0 {
        (*p).nProgress = 0;
    }
    let (iBegin, sBegin) = beginTimer();
    let mut zErrMsg = String::new();
    rc = shell_exec(p, &zSql, &mut zErrMsg);
    endTimer(iBegin, sBegin);

    if rc != 0 || !zErrMsg.is_empty() {
        let zErrorTail;
        let zErrorType: &'static str;

        if zErrMsg.is_empty() {
            zErrorType = "Error";
            zErrorTail = cstr_to_string!(sqlite3_errmsg((*p).db));
        } else if zErrMsg.starts_with("in prepare, ") {
            zErrorType = "Parse error";
            zErrorTail = Cow::from(&zErrMsg[12..]);
        } else if zErrMsg.starts_with("stepping, ") {
            zErrorType = "Runtime error";
            zErrorTail = Cow::from(&zErrMsg[10..]);
        } else {
            zErrorType = "Error";
            zErrorTail = Cow::from(zErrMsg);
        }

        let zPrefix = if stdin_is_interactive == false {
            format!("{} near line {}", zErrorType, startline)
        } else {
            format!("{}:", zErrorType)
        };

        eprintln!("{} {}", zPrefix, zErrorTail);
        // sqlite3_free(zErrMsg as *mut libc::c_void);
        // zErrMsg = std::ptr::null_mut();
        return 1;
    } else {
        if (*p).shellFlgs & 0x20 != 0 {
            println!(
                "changes: {}   total_changes: {}",
                sqlite3_changes64((*p).db),
                sqlite3_total_changes64((*p).db),
            );
        }
    }
    return 0;
}

use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

#[derive(Completer, Helper, Highlighter, Hinter)]
struct InputValidator {}

impl Validator for InputValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        use ValidationResult::{Incomplete, Invalid, Valid};
        let input = ctx.input();
        let result = if !input.starts_with("SELECT") {
            Invalid(Some(" --< Expect: SELECT stmt".to_owned()))
        } else if !input.ends_with(';') {
            Incomplete
        } else {
            Valid(None)
        };
        Ok(result)
    }
}

unsafe fn process_input(mut p: *mut ShellState) -> i32 {
    let mut zSql = String::new();
    let mut nLine: i32 = 0;
    let mut nSql: i32 = 0;
    let mut rc: i32 = 0;
    let mut errCnt: i32 = 0;
    let mut startline: i32 = 0;
    let mut qss: QuickScanState = QSS_Start;
    if (*p).inputNesting == 25 {
        eprintln!("Input nesting limit ({}) reached at line {}. Check recursion", 25, (*p).lineno);
        return 1;
    }
    (*p).inputNesting += 1;
    (*p).lineno = 0;

    let mut rl = rustyline::Editor::<()>::new().unwrap();
    loop {
        if (errCnt == 0 || bail_on_error == false || stdin_is_interactive == true) == false {
            break;
        }

        let zPrompt = if nSql > 0 { &(*p).continuePrompt } else { &(*p).mainPrompt };

        let readline = rl.readline(&zPrompt);
        match readline {
            Ok(zLine) => {
                if zLine.is_empty() {
                    continue;
                }

                (*p).lineno += 1;
                if (zLine.starts_with('.') || zLine.starts_with('#')) && nSql == 0 {
                    if (*p).shellFlgs & 0x40 != 0 {
                        println!("{}", zLine);
                    }
                    if zLine.starts_with('.') {
                        rc = do_meta_command(&zLine, p);
                        if rc == 2 {
                            break;
                        }
                        if rc != 0 {
                            errCnt += 1;
                        }
                    }
                    qss = QSS_Start;
                } else {
                    nLine = zLine.len() as i32;
                    if nSql == 0 {
                        let zLine = zLine.trim_start();
                        zSql = format!("{}", zLine);
                        startline = (*p).lineno;
                        nSql = zSql.len() as i32;
                    } else {
                        zSql.push('\n');
                        zSql.push_str(&zLine);
                        nSql += nLine + 1;
                    }

                    if nSql != 0 && sqrite::is_complete(&zSql) {
                        errCnt += runOneSqlLine(p, &zSql, startline);
                        nSql = 0;
                        if (*p).outCount != 0 {
                            output_reset(p);
                            (*p).outCount = 0;
                        } else {
                            clearTempFile(&mut *p);
                        }
                        (*p).bSafeMode = (*p).bSafeModePersist;
                        qss = QSS_Start;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                // println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    // while errCnt == 0 || bail_on_error == 0 || stdin_is_interactive == true {
    //     zLine = one_input_line(&mut zLine, nSql > 0);
    //     if zLine.is_empty() {
    //         if stdin_is_interactive == true {
    //             println!();
    //         }
    //         break;
    //     } else {
    //         if seenInterrupt != 0 {
    //             ::std::ptr::write_volatile(&mut seenInterrupt as *mut i32, 0);
    //         }
    //         (*p).lineno += 1;
    //         if qss & QSS_CharMask == QSS_Start
    //             && line_is_command_terminator(&zLine)
    //             && line_is_complete(&zSql)
    //         {
    //             zLine = ";".to_string();
    //         }
    //         qss = quickscan(&zLine, qss);
    //         if (qss & !(QSS_EndingSemi ) == QSS_Start)  && nSql == 0 {
    //             println!("masuk sini");
    //             if (*p).shellFlgs & 0x40  != 0  {
    //                 println!("{}", zLine);
    //             }
    //             qss = QSS_Start;
    //         } else if !zLine.is_empty() && (zLine.starts_with('.') || zLine.starts_with('#')) && nSql == 0 {
    //             println!("masuk sonoi");
    //             if (*p).shellFlgs & 0x40 != 0  {
    //                 println!("{}", zLine);
    //             }
    //             if zLine.starts_with('.') {
    //                 rc = do_meta_command(&zLine, p);
    //                 if rc == 2 {
    //                     break;
    //                 }
    //                 if rc != 0 {
    //                     errCnt += 1;
    //                 }
    //             }
    //             qss = QSS_Start;
    //         } else {
    //             println!("masuk else");
    //         }
    //     }
    // }
    if nSql != 0 {
        errCnt += runOneSqlLine(p, &zSql, startline);
    }
    (*p).inputNesting -= 1;
    return (errCnt > 0) as i32;
}

unsafe fn process_sqliterc(mut p: *mut ShellState, sqliterc_override: Option<&str>) {
    let savedLineno = (*p).lineno;

    let mut home_dir = match std::env::home_dir() {
        None => {
            eprintln!("-- warning: cannot find home directory; cannot read ~/.sqliterc");
            return;
        }
        Some(path) => path,
    };

    let sqliterc = match sqliterc_override {
        None => {
            home_dir.push(".sqliterc");
            home_dir.as_path()
        }
        Some(text) => Path::new(text),
    };

    let mut handle = File::open(sqliterc);
    match handle {
        Ok(_) => {
            if stdin_is_interactive == true {
                eprintln!("-- Loading resources from {}", sqliterc.to_string_lossy());
            }
            if process_input(p) != 0 && bail_on_error {
                std::process::exit(1);
            }
        }
        Err(_) => {
            if sqliterc_override.is_some() {
                eprintln!("cannot open: \"{}\"", sqliterc.to_string_lossy());
                if bail_on_error {
                    std::process::exit(1);
                }
            }
        }
    }

    (*p).lineno = savedLineno;
}

fn usage(showDetail: bool) -> ! {
    let arg0 = std::env::args().next().unwrap();
    eprintln!("Usage: {arg0} [OPTIONS] FILENAME [SQL]\nFILENAME is the name of an SQLite database. A new database is created\nif the file does not previously exist.");
    if showDetail {
        let zOptions = "   -append              append the database to the end of the file\n   -ascii               set output mode to 'ascii'\n   -bail                stop after hitting an error\n   -batch               force batch I/O\n   -box                 set output mode to 'box'\n   -column              set output mode to 'column'\n   -cmd COMMAND         run \"COMMAND\" before reading stdin\n   -csv                 set output mode to 'csv'\n   -deserialize         open the database using sqlite3_deserialize()\n   -echo                print commands before execution\n   -init FILENAME       read/process named file\n   -[no]header          turn headers on or off\n   -help                show this message\n   -html                set output mode to HTML\n   -interactive         force interactive I/O\n   -json                set output mode to 'json'\n   -line                set output mode to 'line'\n   -list                set output mode to 'list'\n   -lookaside SIZE N    use N entries of SZ bytes for lookaside memory\n   -markdown            set output mode to 'markdown'\n   -maxsize N           maximum size for a --deserialize database\n   -memtrace            trace all memory allocations and deallocations\n   -mmap N              default mmap size set to N\n   -newline SEP         set output row separator. Default: '\\n'\n   -nofollow            refuse to open symbolic links to database files\n   -nonce STRING        set the safe-mode escape nonce\n   -nullvalue TEXT      set text string for NULL values. Default ''\n   -pagecache SIZE N    use N slots of SZ bytes each for page cache memory\n   -quote               set output mode to 'quote'\n   -readonly            open the database read-only\n   -safe                enable safe-mode\n   -separator SEP       set output column separator. Default: '|'\n   -stats               print memory stats before each finalize\n   -table               set output mode to 'table'\n   -tabs                set output mode to 'tabs'\n   -version             show SQLite version\n   -vfs NAME            use NAME as the default VFS";
        eprintln!("OPTIONS include:\n{zOptions}");
    } else {
        eprintln!("Use the -help option for additional information");
    }
    std::process::exit(1);
}

/// Internal check:  Verify that the SQLite is uninitialized.  Print a error message if it is initialized.
fn verify_uninitialized() {
    let result = unsafe { sqlite3_config(-1) };
    if result == 21 {
        println!("WARNING: attempt to configure SQLite after initialization");
    }
}

unsafe fn main_init(mut data: *mut ShellState) {
    (*data).mode = 2;
    (*data).cMode = 2;
    (*data).normalMode = 2;
    (*data).autoExplain = 1;
    (*data).pAuxDb = &mut *((*data).aAuxDb).as_mut_ptr().offset(0) as *mut AuxDb;
    (*data).colSeparator = SmolStr::new_inline("|");
    (*data).rowSeparator = SmolStr::new_inline("\n");
    (*data).showHeader = 0;
    (*data).shellFlgs = 0x2 as u32;
    verify_uninitialized();
    sqlite3_config(17, 1);
    // NOTE: don't remove cast, removing it causing segmentation fault when running query
    sqlite3_config(16, shellLog as unsafe extern "C" fn(*mut libc::c_void, i32, *const i8), data);
    sqlite3_config(2);

    (*data).mainPrompt = SmolStr::new_inline("sqlite> ");
    (*data).continuePrompt = SmolStr::new_inline("   ...> ");
}

fn printBold(zText: &str) {
    print!("\x1B[1m{zText}\x1B[0m");
}

fn cmdline_option_value(args: &Vec<String>, i: i32) -> String {
    if i == (args.len() - 1) as i32 {
        eprintln!("{}: Error: missing argument to {}", args[0], args[args.len() - 1]);
        std::process::exit(1);
    }
    return args[i as usize].clone();
}

unsafe fn main_0() -> i32 {
    let args: Vec<String> = {
        let mut args: Vec<String> = std::env::args().collect();
        args.push(String::new());
        args
    };
    let argc = args.len() as i32;

    let mut zErrMsg = String::new();
    let mut zInitFile: Option<&str> = None;
    let mut i: i32 = 0;
    let mut rc: i32 = 0;
    let mut warnInmemoryDb: i32 = 0;
    let mut readStdin: bool = true;
    let mut nCmd: i32 = 0;
    let mut azCmd: Vec<String> = Vec::new();
    let mut zVfs = String::new();

    unsafe {
        setvbuf(stderr, std::ptr::null_mut(), 2, 0);
        stdin_is_interactive = isatty(0) != 0;
        stdout_is_console = isatty(1);
    }

    if std::env::var("SQLITE_DEBUG_BREAK").is_ok() {
        if isatty(0) != 0 && isatty(2) != 0 {
            eprintln!("attach debugger to process {} and press any key to continue", std::process::id());
            fgetc(stdin);
        } else {
            raise(5);
        }
    }

    if strncmp(
        sqlite3_sourceid(),
        b"2022-09-05 11:02:23 4635f4a69c8c2a8df242b384a992aea71224e39a2ccab42d8c0b0602f1e826e8\0" as *const u8 as *const i8,
        60,
    ) != 0
    {
        eprintln!(
            "SQLite header and source version mismatch\n{}\n{}",
            cstr_to_string!(sqlite3_sourceid()),
            "2022-09-05 11:02:23 4635f4a69c8c2a8df242b384a992aea71224e39a2ccab42d8c0b0602f1e826e8"
        );
        std::process::exit(1);
    }

    let mut data: ShellState = ShellState::default();
    main_init(&mut data);

    assert!(args.len() >= 1 && args[0].is_empty() == false);

    let argsv0 = args[0].clone();
    signal(2, Some(interrupt_handler));
    verify_uninitialized();
    i = 1;
    let mut temp_zInitFile = String::new(); // temporary hack to allow store zInitFile
    while i < argc {
        let zz = args[i as usize].clone();

        if zz.starts_with('-') == false {
            if ((*(data.aAuxDb).as_mut_ptr()).zDbFilename).is_empty() {
                (*(data.aAuxDb).as_mut_ptr()).zDbFilename = zz.to_string();
            } else {
                readStdin = false;
                nCmd += 1;
                azCmd.push(zz.clone());
            }
        }

        let zz = if zz.starts_with("--") {
            zz.strip_prefix('-').unwrap().to_string()
        } else {
            zz
        };

        if zz == "-separator" || zz == "-nullvalue" || zz == "-newline" || zz == "-cmd" {
            i += 1;
            cmdline_option_value(&args, i);
        } else if zz == "-init" {
            i += 1;
            temp_zInitFile = cmdline_option_value(&args, i);
            zInitFile = Some(&temp_zInitFile);
        } else if zz == "-batch" {
            stdin_is_interactive = false;
        } else if zz == "-heap" {
            i += 1;
            cmdline_option_value(&args, i);
        } else if zz == "-pagecache" {
            let mut n: i64 = 0;
            let mut sz: i64 = 0;
            i += 1;
            sz = integerValue(&cmdline_option_value(&args, i));
            sz = sz.clamp(0, 70_000);
            i += 1;
            n = integerValue(&cmdline_option_value(&args, i));
            if sz > 0 && n > 0 && 0xffffffffffff as i64 / sz < n {
                n = 0xffffffffffff as i64 / sz;
            }
            sqlite3_config(
                7 as i32,
                if n > 0 && sz > 0 {
                    malloc((n * sz) as u64)
                } else {
                    std::ptr::null_mut()
                },
                sz,
                n,
            );
            data.shellFlgs |= 0x1;
        } else if zz == "-lookaside" {
            let mut n_0: i32 = 0;
            let mut sz_0: i32 = 0;
            i += 1;
            sz_0 = integerValue(&cmdline_option_value(&args, i)) as i32;
            if sz_0 < 0 {
                sz_0 = 0;
            }
            i += 1;
            n_0 = integerValue(&cmdline_option_value(&args, i)) as i32;
            if n_0 < 0 {
                n_0 = 0;
            }
            sqlite3_config(13, sz_0, n_0);
            if sz_0 * n_0 == 0 {
                data.shellFlgs &= !(0x2) as u32;
            }
        } else if zz == "-threadsafe" {
            i += 1;
            match integerValue(&cmdline_option_value(&args, i)) {
                0 => sqlite3_config(1),
                2 => sqlite3_config(2),
                _ => sqlite3_config(3),
            };
        } else if zz == "-mmap" {
            i += 1;
            let sz_1: i64 = integerValue(&cmdline_option_value(&args, i));
            sqlite3_config(22, sz_1, sz_1);
        } else if zz == "-vfs" {
            i += 1;
            zVfs = cmdline_option_value(&args, i);
        } else if zz == "-append" {
            data.openMode = 2;
        } else if zz == "-deserialize" {
            data.openMode = 5;
        } else if zz == "-maxsize" && (i + 1) < argc {
            i += 1;
            data.szMax = integerValue(&cmdline_option_value(&args, i));
        } else if zz == "-readonly" {
            data.openMode = 4;
        } else if zz == "-nofollow" {
            data.openFlags = 0x1000000;
        } else if zz == "-memtrace" {
            sqlite3MemTraceActivate(stderr);
        } else if zz == "-bail" {
            bail_on_error = true;
        } else if zz == "-nonce" {
            i += 1;
            data.zNonce = args[i as usize].clone();
        } else if zz == "-safe" {
        }
        i += 1;
    }
    verify_uninitialized();
    sqlite3_initialize();
    if !zVfs.is_empty() {
        let mut pVfs: *mut sqlite3_vfs = sqlite3_vfs_find(to_cstring!(zVfs));
        if !pVfs.is_null() {
            sqlite3_vfs_register(pVfs, 1);
        } else {
            eprintln!("no such VFS: \"{}\"", args[i as usize]);
            std::process::exit(1);
        }
    }
    if ((*data.pAuxDb).zDbFilename).is_empty() {
        (*data.pAuxDb).zDbFilename = ":memory:".to_string();
        warnInmemoryDb = (argc == 1) as i32;
    }

    sqlite3_appendvfs_init(0 as *mut sqlite3, 0 as *mut *mut i8, 0 as *const sqlite3_api_routines);
    let path = std::path::Path::new(&*(*data.pAuxDb).zDbFilename);
    if path.exists() {
        open_db(&mut data, 0);
    }
    process_sqliterc(&mut data, zInitFile);
    i = 1;
    while i < argc {
        let mut zz_0 = args[i as usize].clone();
        if zz_0.starts_with('-') == false {
            i += 1;
            continue;
        }

        zz_0 = if zz_0.starts_with("--") {
            zz_0.strip_prefix('-').unwrap().to_string()
        } else {
            zz_0
        };

        if zz_0 == "-init" {
            i += 1;
        } else if zz_0 == "-html" {
            data.mode = 4;
        } else if zz_0 == "-list" {
            data.mode = 2;
        } else if zz_0 == "-quote" {
            data.mode = 6;
            data.colSeparator = SmolStr::new_inline(",");
            data.rowSeparator = SmolStr::new_inline("\n");
        } else if zz_0 == "-line" {
            data.mode = 0;
        } else if zz_0 == "-column" {
            data.mode = 1;
        } else if zz_0 == "-json" {
            data.mode = 13;
        } else if zz_0 == "-markdown" {
            data.mode = 14;
        } else if zz_0 == "-table" {
            data.mode = 15;
        } else if zz_0 == "-box" {
            data.mode = 16;
        } else if zz_0 == "-csv" {
            data.mode = 8;
            data.colSeparator = SmolStr::new_inline(",");
        } else if zz_0 == "-append" {
            data.openMode = 2 as u8;
        } else if zz_0 == "-deserialize" {
            data.openMode = 5 as u8;
        } else if zz_0 == "-maxsize" && (i + 1) < argc {
            i += 1;
            data.szMax = integerValue(&args[i as usize]);
        } else if zz_0 == "-readonly" {
            data.openMode = 4 as u8;
        } else if zz_0 == "-nofollow" {
            data.openFlags |= 0x1000000;
        } else if zz_0 == "-ascii" {
            data.mode = 10;
            data.colSeparator = SmolStr::new_inline("\x1F");
            data.rowSeparator = SmolStr::new_inline("\x1E");
        } else if zz_0 == "-tabs" {
            data.mode = 2;
            data.colSeparator = SmolStr::new_inline("\t");
            data.rowSeparator = SmolStr::new_inline("\n");
        } else if zz_0 == "-separator" {
            i += 1;
            data.colSeparator = SmolStr::new(&cmdline_option_value(&args, i));
        } else if zz_0 == "-newline" {
            i += 1;
            data.rowSeparator = SmolStr::new(&cmdline_option_value(&args, i));
        } else if zz_0 == "-nullvalue" {
            i += 1;
            data.nullValue = SmolStr::new(&cmdline_option_value(&args, i));
        } else if zz_0 == "-header" {
            data.showHeader = 1;
            data.shellFlgs |= 0x80;
        } else if zz_0 == "-noheader" {
            data.showHeader = 0;
            data.shellFlgs |= 0x80;
        } else if zz_0 == "-echo" {
            data.shellFlgs |= 0x40;
        } else if zz_0 == "-eqp" {
            data.autoEQP = 1;
        } else if zz_0 == "-eqpfull" {
            data.autoEQP = 3;
        } else if zz_0 == "-stats" {
            data.statsOn = 1;
        } else if zz_0 == "-scanstats" {
            data.scanstatsOn = 1;
        } else if zz_0 == "-backslash" {
            data.shellFlgs |= 0x4 as i32 as u32;
        } else if zz_0 != "-bail" {
            if zz_0 == "-version" {
                println!("{} {}", cstr_to_string!(sqlite3_libversion()), cstr_to_string!(sqlite3_sourceid()),);
                return 0;
            } else if zz_0 == "-interactive" {
                stdin_is_interactive = true;
            } else if zz_0 == "-batch" {
                stdin_is_interactive = false;
            } else if zz_0 == "-heap" {
                i += 1;
            } else if zz_0 == "-pagecache" {
                i += 2;
            } else if zz_0 == "-lookaside" {
                i += 2;
            } else if zz_0 == "-threadsafe" {
                i += 2;
            } else if zz_0 == "-nonce" {
                i += 2;
            } else if zz_0 == "-mmap" {
                i += 1;
            } else if zz_0 == "-memtrace" {
                i += 1;
            } else if zz_0 == "-vfs" {
                i += 1;
            } else if zz_0 == "-help" {
                usage(true);
            } else if zz_0 == "-cmd" {
                if i == argc - 1 {
                    break;
                }
                i += 1;

                zz_0 = cmdline_option_value(&args, i);
                if zz_0.starts_with('.') {
                    rc = do_meta_command(&zz_0, &mut data);
                    if rc != 0 && bail_on_error {
                        return if rc == 2 { 0 } else { rc };
                    }
                } else {
                    open_db(&mut data, 0);
                    rc = shell_exec(&mut data, &zz_0, &mut zErrMsg);
                    if !zErrMsg.is_empty() {
                        eprintln!("Error: {}", zErrMsg);
                        if bail_on_error {
                            return if rc != 0 { rc } else { 1 };
                        }
                    } else if rc != 0 {
                        eprintln!("Error: unable to process SQL \"{}\"", zz_0.clone());
                        if bail_on_error {
                            return rc;
                        }
                    }
                }
            } else if zz_0 == "-safe" {
                data.bSafeModePersist = 1;
                data.bSafeMode = data.bSafeModePersist;
            } else {
                eprintln!("{}: Error: unknown option: {}", argsv0, zz_0);
                eprintln!("Use -help for a list of options.");
                return 1;
            }
        }
        data.cMode = data.mode;
        i += 1;
    }

    if readStdin == false {
        for i in 0..nCmd {
            if azCmd[i as usize].starts_with('.') {
                rc = do_meta_command(&azCmd[i as usize], &mut data);
                if rc != 0 {
                    return if rc == 2 { 0 } else { rc };
                }
            } else {
                open_db(&mut data, 0);
                rc = shell_exec(&mut data, &azCmd[i as usize], &mut zErrMsg);
                if !zErrMsg.is_empty() || rc != 0 {
                    if !zErrMsg.is_empty() {
                        eprintln!("Error: {}", zErrMsg);
                    } else {
                        eprintln!("Error: unable to process SQL: {}", &azCmd[i as usize]);
                    }

                    return if rc != 0 { rc } else { 1 };
                }
            }
        }
    } else if stdin_is_interactive == true {
        println!(
            "SQLite version {} {}\nEnter \".help\" for usage hints",
            cstr_to_string!(sqlite3_libversion()),
            cstr_to_string!(sqlite3_sourceid()),
        );
        if warnInmemoryDb != 0 {
            print!("Connected to a ");
            printBold("transient in-memory database");
            println!("\nUse \".open FILENAME\" to reopen on a persistent database.");
        }

        rc = process_input(&mut data);
    } else {
        rc = process_input(&mut data);
    }
    set_table_name(&mut data, "");
    if !(data.db).is_null() {
        close_db(data.db);
    }
    for auxDb in &data.aAuxDb {
        if !(auxDb.db).is_null() {
            close_db(auxDb.db);
        }
    }

    output_reset(&mut data);
    data.doXdgOpen = 0;
    clearTempFile(&mut data);

    return rc;
}
pub fn main() {
    unsafe { ::std::process::exit(main_0()) }
}
