import type { AuthUser } from "./auth/auth.svelte";
import { UserFlag } from "./auth/userFlags";

// gosh this method is a mess dumb bird. it does work tho... yippie
export const getRedirectUrl = (user: AuthUser | null, url: URL): string | null => {
    const path = url.pathname;
    const isLoggedIn = user != null;

    const sudoPaths = path.startsWith("/auth/sudo");
    const completeSignupPaths = path.startsWith("/signup/complete"); // ew forced hordering
    const unauthedPaths =
        path.startsWith("/login") || (path.startsWith("/signup") && !completeSignupPaths);
    const iDontCarePaths = path == "/" || (path.startsWith("/auth") && !sudoPaths);
    const authedPaths = sudoPaths || path.startsWith("/me");

    if (isLoggedIn && !user.flags.has(UserFlag.HasSetName) && !completeSignupPaths) {
        return `/signup/complete?redirect=${encodeURIComponent(path)}`;
    } else if (isLoggedIn && user.flags.has(UserFlag.HasSetName) && completeSignupPaths) {
        return "/me";
    }

    if (isLoggedIn && unauthedPaths) {
        return "/me";
    }

    if (!isLoggedIn && !unauthedPaths && !iDontCarePaths && authedPaths) {
        return `/login?redirect=${encodeURIComponent(path)}`;
    }

    return null;
};
