import queryClient from "$lib/api/tanstackClient";

import { isOk } from "../api/ignoreThisPlease";
import type { User } from "../api/model";
import {
    currentUserInfo,
    getCurrentUserInfoQueryKey,
    type currentUserInfoResponse
} from "../api/user/user";
import { UserFlags } from "./userFlags";

export interface AuthUser extends Omit<User, "flags"> {
    flags: UserFlags;
}

/// its the auth context from the backend, but in the frontend!
class AuthState {
    private SESSION_KEY = "meow.";
    private PENDING_AUTH_EMAIL_SESS_KEY = this.SESSION_KEY + "pendingAuthEmail";

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
    private _pendingAuthEmail = $state<string | undefined>(this.pendingAuthEmailState());

    private pendingAuthEmailState(): string | undefined {
        try {
            const data = sessionStorage.getItem(this.PENDING_AUTH_EMAIL_SESS_KEY);
            return data ? JSON.parse(data) : undefined;
        } catch {
            return undefined;
        }
    }

    public get pendingAuthEmail(): string | undefined {
        return this._pendingAuthEmail;
    }

    public set pendingAuthEmail(email: string | undefined) {
        this._pendingAuthEmail = undefined;
        try {
            if (email == undefined) {
                sessionStorage.removeItem(this.PENDING_AUTH_EMAIL_SESS_KEY);
            } else {
                sessionStorage.setItem(this.PENDING_AUTH_EMAIL_SESS_KEY, JSON.stringify(email));
            }
        } catch {
            console.error("failed setting the pending authentication email :(");
        }
    }

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

    /** Sync the user state from the tanstack query stuffies */
    public async sync(response: currentUserInfoResponse | undefined, isPending: boolean = false) {
        console.log("updating auth state");
        if (!response) {
            console.warn("did not update auth state because response isnt responding");
            return;
        }

        // if (this.loading) {
        //     console.warn("auth state update ignored because it's already loading");
        //     return;
        // }

        this.loading = isPending;
        if (isOk(response)) {
            console.log("user has a session");
            let flags = new UserFlags(response.data.flags);
            this.user = { ...response.data, flags };
            this.pendingAuthEmail = undefined;

            if (!this.alreadyPrinted) {
                console.log("user state: ", {
                    ...$state.snapshot(this.user),
                    flags: this.user.flags.toStrings()
                });
                this.alreadyPrinted = true;
            }
        } else {
            this.user = null;
            console.log("user doesn't have a session");
        }
    }

    /** Fetches the user info and syncs it to the auth state AND the tanstack queries */
    public async fetchAndSynctan(fetcher: typeof globalThis.fetch) {
        console.log("fetching and syncing auth state");
        if (this.loading) {
            console.warn("auth state update ignored because it's already loading");
            return;
        }
        this.loading = true;
        try {
            const response = await currentUserInfo(undefined, fetcher);
            queryClient.setQueryData(getCurrentUserInfoQueryKey(), response);
            await this.sync(response);
        } catch (e) {
            this.user = null;
            console.error("something went wrong while fetching the auth state:", e);
        } finally {
            this.loading = false;
        }
        console.log("auth state should be updated");
    }
}

export const auth = new AuthState(); // ew, forced ordering.
