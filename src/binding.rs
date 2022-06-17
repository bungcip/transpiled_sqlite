pub mod string {
    use crate::src::sqlite3;
    use std::ffi::CString;

    ///
    /// CAPI3REF: String Globbing
    ///
    /// ^The [sqlite3_strglob(P,X)] interface returns zero if and only if
    /// string X matches the [GLOB] pattern P.
    /// ^The definition of [GLOB] pattern matching used in
    /// [sqlite3_strglob(P,X)] is the same as for the "X GLOB P" operator in the
    /// SQL dialect understood by SQLite.  ^The [sqlite3_strglob(P,X)] function
    /// is case sensitive.
    ///
    ///Note that this routine returns zero on a match and non-zero if the strings
    ///do not match, the same as [sqlite3_stricmp()] and [sqlite3_strnicmp()].
    ///
    /// See also: [sqlite3_strlike()].
    pub fn strglob(glob: &str, content: &str) -> bool {
        unsafe { sqlite3::sqlite3_strglob(CString::new(glob).unwrap().as_ptr(), CString::new(content).unwrap().as_ptr()) == 0 }
    }

    /*
     ** CAPI3REF: String LIKE Matching
     *
     ** ^The [sqlite3_strlike(P,X,E)] interface returns zero if and only if
     ** string X matches the [LIKE] pattern P with escape character E.
     ** ^The definition of [LIKE] pattern matching used in
     ** [sqlite3_strlike(P,X,E)] is the same as for the "X LIKE P ESCAPE E"
     ** operator in the SQL dialect understood by SQLite.  ^For "X LIKE P" without
     ** the ESCAPE clause, set the E parameter of [sqlite3_strlike(P,X,E)] to 0.
     ** ^As with the LIKE operator, the [sqlite3_strlike(P,X,E)] function is case
     ** insensitive - equivalent upper and lower case ASCII characters match
     ** one another.
     **
     ** ^The [sqlite3_strlike(P,X,E)] function matches Unicode characters, though
     ** only ASCII characters are case folded.
     **
     ** Note that this routine returns zero on a match and non-zero if the strings
     ** do not match, the same as [sqlite3_stricmp()] and [sqlite3_strnicmp()].
     **
     ** See also: [sqlite3_strglob()].
     */
    pub fn strlike(glob: &str, content: &str, esc: char) -> bool {
        unsafe {
            sqlite3::sqlite3_strlike(
                CString::new(glob).unwrap().as_ptr(),
                CString::new(content).unwrap().as_ptr(),
                esc as u32,
            ) == 0
        }
    }

    /*
     ** Return TRUE if the given SQL string ends in a semicolon.
     **
     ** Special handling is require for CREATE TRIGGER statements.
     ** Whenever the CREATE TRIGGER keywords are seen, the statement
     ** must end with ";END;".
     **
     */
    pub fn is_complete(content: &str) -> bool {
        unsafe { sqlite3::sqlite3_complete(CString::new(content).unwrap().as_ptr()) != 0 }
    }
}
