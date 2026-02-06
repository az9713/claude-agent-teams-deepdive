// A sample TypeScript file for testing TODO detection

interface User {
    name: string;
    // TODO(alice, #456, p:critical): Add email validation
    email: string;
}

// FIXME: Type assertion is unsafe here
function getUser(): User {
    return {} as User;
}
