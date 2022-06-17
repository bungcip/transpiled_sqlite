#!/usr/bin/env python3

## I don't want to write parser & lexer for rust in python
## so I just create this simple helper to write the refactorer tools


class MuncherError(Exception): pass

def expect(buffer: str, pos: int, ch: str) -> int:
    if buffer[pos] == ch:
        return pos + 1
    else:
        raise MuncherError("expected ch: {}, but got: {} instead.".format(ch, buffer[pos]))


def backward_expect(buffer: str, pos: int, ch: str) -> int:
    """ go backward for position (whitespace is ignored), if ch found return the position, exceptio otherwise """

    endpos = pos
    length = len(ch)
    while True:
        pos -= 1

        # print("token {}: {}".format(pos, buffer[pos]))

        if buffer[pos] in [' ', '\r', '\t', '\n']:
            continue

        loc = buffer.find(ch, pos, endpos)
        if loc != -1:
            return loc
        elif length > 1:
            length -= 1
        else:
            raise MuncherError("backward expected ch: '{}', but got: '{}' instead.".format(ch, buffer[pos:endpos]))

def backward_until(buffer: str, pos: int, ch: str) -> int:
    """ go backward for position until found ch. otherwise raised exception."""
    endpos = pos
    length = len(ch)
    while pos > 0:
        pos -= length
        loc = buffer.find(ch, pos, endpos)
        if loc != -1:
            return loc
        else:
            continue

    raise MuncherError("backward_until: cannot find '{}'".format(ch))

def find_matching_pair_span(buffer: str, pos: int, begin_token: str, end_token: str):
    """ find matching balanced pair. first token must begin with <begin_token>"""
    pos = begin_pos = expect(buffer, pos, begin_token)

    # stop until matching ')
    count = 1
    while count > 0:
        if buffer[pos] == begin_token:
            count += 1
        elif buffer[pos] == end_token:
            count -= 1
        pos += 1

    return (begin_pos, pos)

def replace_span(buffer: str, span: tuple, replace: str):
    buffer = buffer[:span[0]] + replace + buffer[span[1]:]
    return buffer


