<script>
    import Button from "$comps/button.svelte";
    import AuditLogItem from "$comps/items/auditLogItem.svelte";
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
</script>

<SettingsPanel title="Profile" description="Its your identity when you sign in to other apps!">
    <div class="flex gap-4 justify-between align-center baba">
        <div class="avatar aa">?</div>

        <div class="upload-zone">
            <p><a href="#main">Upload image</a> or drag and drop</p>
            <p>PNG, JPEG or WEBP (max. 256x256, 256KiB)</p>
        </div>
    </div>
    <div class="flex flex-col gap-2 text-start">
        <label>Name</label>
        <input class="input" value="Cow69" placeholder="Name" type="text" />
    </div>
    <div class="flex flex-col gap-2 text-start">
        <label>Email address</label>
        <div class="flex input input--proxy">
            <input
                class="input"
                value="meow@meow.com"
                placeholder="Email address"
                readonly
                type="email"
            />
            <a href="#main">Meow</a>
        </div>
    </div>
    <Button primary>Save changes</Button>
</SettingsPanel>

<SettingsPanel
    title="Audit log"
    description="A little trail of your recent activity on your account"
>
    <LogItem.Container>
        <AuditLogItem2 title="Created session" tag="Auth" who="you" when="2h ago">
            <span>hi this is metadata or just uselss</span>
        </AuditLogItem2>
        <AuditLogItem2 />
        <AuditLogItem2 />
        <AuditLogItem2 />
        <AuditLogItem2 />
        <AuditLogItem2 />
        <AuditLogItem2 />
        <AuditLogItem2 />
    </LogItem.Container>
    <Button>View all</Button>
</SettingsPanel>

<SettingsPanel
    title="Delete account"
    description="Delete your account and everything associated with it, including authorized apps and created apps."
    negative
>
    <Button negative>Delete account</Button>
</SettingsPanel>

<style lang="scss">
    .aa {
        position: relative;
        width: 96px;
        height: 96px;
        aspect-ratio: 1;
        /* border: 1px solid var(--color-iron-dark); */

        &::before {
            position: absolute;
            content: "";
            width: 100%;
            height: 100%;
            border-radius: 50%;
            border: 1px solid var(--color-iron-darkest);
        }
    }

    .upload-zone {
        padding-block: calc(var(--spacing) * 4);
        padding-inline: calc(var(--spacing) * 6);
        border-radius: var(--typical-radius);
        border: 1px dashed var(--color-iron-dark);

        justify-content: center;
        display: flex;
        flex-direction: column;

        p {
            font-size: var(--text-small);
        }
    }

    // .audit-log {
    //     display: flex;
    //     border-bottom: 1px solid var(--color-iron-dark);
    //     border-inline: 1px solid var(--color-iron-dark);

    //     padding-block: calc(var(--spacing) * 3);
    //     padding-inline: calc(var(--spacing) * 5);
    //     align-items: center;
    //     gap: calc(var(--spacing) * 4);
    //     text-align: start;

    //     &:first-child {
    //         border-top-left-radius: var(--typical-radius);
    //         border-top-right-radius: var(--typical-radius);
    //         border-top: 1px solid var(--color-iron-dark);
    //     }

    //     &:last-child {
    //         border-bottom-left-radius: var(--typical-radius);
    //         border-bottom-right-radius: var(--typical-radius);
    //         border-bottom: 1px solid var(--color-iron-dark);
    //     }

    //     .content {
    //         display: flex;
    //         flex: 1;
    //         min-inline-size: 0;
    //         flex-direction: column;
    //     }
    // }
    // .audit-log .content p,
    // .audit-log .content c {
    //     overflow-wrap: anywhere; // or word-break: break-word;
    // }

    @media (max-width: 768px) {
        .baba {
            flex-direction: column;
            align-items: center;
        }
        // .audit-log {
        //     flex-wrap: wrap;
        // }
    }
</style>
