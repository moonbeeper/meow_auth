import { isOk } from "../api/ignoreThisPlease";
import type { User } from "../api/model";
import { currentUserInfo } from "../api/user/user";
import { UserFlags } from "./userFlags";

export interface AuthUser extends Omit<User, "flags"> {
    flags: UserFlags;
}

/// its the auth context from the backend, but in the frontend!
class AuthState {
    // if we already printed the user state (or else we reprint it every time we invalidate or change routes)
    private alreadyPrinted: boolean = false;
    // private lastFetchedAt: number = 0;
    // private static readonly fetch_interval: number = 1000; // 1 second
    public user: AuthUser | null = $state<AuthUser | null>(null);
    public loading: boolean = $state(false);
    /** Be aware! Looses state when page is reloaded. (I could use session sotrage with a expiry lol and boom solved)
     *
     * This is used to store the email of the user that is currently in the process of being authenticated (otp)
     */
    public pendingAuthEmail = $state<string | undefined>(undefined);

    public async update(fetcher: typeof globalThis.fetch) {
        // i hate you preload.
        // const now = Date.now();
        // if (now - this.lastFetchedAt < AuthState.fetch_interval) {
        //     console.warn("didnt update auth state, try again after a second");
        //     return;
        // }
        // this.lastFetchedAt = now;

        console.log("updating auth state");
        if (this.loading) {
            console.warn("auth state update ignored because it's already loading");
            return;
        }
        this.loading = true;
        try {
            const res = await currentUserInfo(undefined, fetcher);
            if (isOk(res)) {
                console.log("user has a session");
                let flags = new UserFlags(res.data.flags);
                this.user = { ...res.data, flags };
                if (!this.alreadyPrinted) {
                    console.log("user state: ", $state.snapshot(this.user));
                    this.alreadyPrinted = true;
                }
                this.pendingAuthEmail = undefined;
            } else {
                this.user = null;
                console.log("user doesn't have a session");
            }
        } catch (err) {
            console.error("something went wrong while updating the auth state: ", err);
            this.user = null;
        } finally {
            this.loading = false;
        }
    }
}

export const auth = new AuthState(); // ew, forced ordering.
