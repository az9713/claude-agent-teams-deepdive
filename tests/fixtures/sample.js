// A sample JavaScript file for testing TODO detection

function main() {
    // TODO: Add input validation
    const x = 42;

    /* FIXME(alice, #456): Memory leak in event handler */
    document.addEventListener('click', () => {});

    // HACK: Workaround for Safari bug
    const ua = navigator.userAgent;
}

/*
 * XXX: This entire function needs a rewrite
 * It's doing too many things at once
 */
function legacy() {}
