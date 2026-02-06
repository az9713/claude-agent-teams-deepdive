// A sample JavaScript file with block comment TODOs

function regular() {
    // TODO: Inline line comment TODO
    const x = 42;
}

/*
 * FIXME(alice, #101): This is inside a
 * multi-line block comment
 */
function broken() {}

/* HACK: Single line block comment */
function workaround() {}

/**
 * BUG(bob, p:critical): Jsdoc-style block comment
 * with a bug report
 */
function buggy() {}

function clean() {
    const notATodo = "TODO in a string, not a comment";
}
