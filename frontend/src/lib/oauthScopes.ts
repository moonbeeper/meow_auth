import { EnumFlags } from "./flags";

export enum OauthScope {
    OpenId = 1 << 0,
    Profile = 1 << 1,
    Email = 1 << 2
}

export class OauthScopes extends EnumFlags<OauthScope> {
    private static readonly allFlags: number[] = [
        OauthScope.OpenId,
        OauthScope.Profile,
        OauthScope.Email
    ];

    private static readonly humanNames: Record<OauthScope, string> = {
        [OauthScope.OpenId]: "smth to do with someone named oidc",
        [OauthScope.Profile]: "Your profile",
        [OauthScope.Email]: "Your email address"
    };

    protected get all(): OauthScope[] {
        return OauthScopes.allFlags;
    }

    protected get names(): Record<OauthScope, string> {
        return OauthScopes.humanNames;
    }

    constructor(public bits: number = 0) {
        super(bits);
    }
}
