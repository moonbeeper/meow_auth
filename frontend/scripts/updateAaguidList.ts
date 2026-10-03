import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";

import { format } from "oxfmt";

const AAGUID_LIST_URL =
    "https://raw.githubusercontent.com/passkeydeveloper/passkey-authenticator-aaguids/refs/heads/main/combined_aaguid.json";

const OUTPUT = resolve("src/lib/passkey/aaguid.ts");

const FILE_HEADER = `// hi, this file is generate by mr script/updateAaguidList. if you want to update this, use the script.

export type AuthenticatorMetadata = {
    name: string;
    /** SVG base64 encoded as a data URI */
    icon_dark: string | undefined;
    /** SVG base64 encoded as a data URI */
    icon_light: string | undefined;
};

export const authenticatorAAGUIDs: Record<string, AuthenticatorMetadata> = {
  `;

const FILE_END = `};`;

async function main() {
    const res = await fetch(AAGUID_LIST_URL);

    if (!res.ok) {
        throw new Error("failed fetching aaguid list :(");
    }

    const data = await res.json();

    const entries = Object.entries(data).map(([aaguid, metadata]) => {
        return ` "${aaguid}": ${JSON.stringify(metadata)},`.split("\n").join("\n");
    });

    const output = `${FILE_HEADER} ${entries.join("\n")} ${FILE_END}`;

    await mkdir(dirname(OUTPUT), { recursive: true });
    await writeFile(OUTPUT, output);

    console.log("done updating");
}

main().catch((e) => {
    console.error(e);
});
