import { EnumFlags } from "./flags";

export enum UserFlag {
    // None = 0,
    /** for big birds like me - moonbeeper :3 */
    SuperAdmin = 1 << 0,
    CannotManageOauthApplications = 1 << 1,
    CannotAuthorizeOauthApplications = 1 << 2,
    CannotModifyName = 1 << 3,
    CannotModifyEmail = 1 << 4,
    /** If this is set, the user has completed their setup process by changing their display name
     *
     * If not, access not granted to anything :(
     */
    HasSetName = 1 << 5
}

export class UserFlags extends EnumFlags<UserFlag> {
    private static readonly allFlags: number[] = [
        UserFlag.SuperAdmin,
        UserFlag.CannotManageOauthApplications,
        UserFlag.CannotAuthorizeOauthApplications,
        UserFlag.CannotModifyName,
        UserFlag.CannotModifyEmail,
        UserFlag.HasSetName
    ];

    private static readonly humanNames: Record<UserFlag, string> = {
        [UserFlag.SuperAdmin]: "Super Admin",
        [UserFlag.CannotManageOauthApplications]: "Cannot manage OAuth apps",
        [UserFlag.CannotAuthorizeOauthApplications]: "Cannot authorize OAuth apps",
        [UserFlag.CannotModifyName]: "Cannot modify name",
        [UserFlag.CannotModifyEmail]: "Cannot modify email",
        [UserFlag.HasSetName]: "Has set display name"
    };

    protected get all(): UserFlag[] {
        return UserFlags.allFlags;
    }

    protected get names(): Record<UserFlag, string> {
        return UserFlags.humanNames;
    }

    constructor(public bits: number = 0) {
        super(bits);
    }
}
