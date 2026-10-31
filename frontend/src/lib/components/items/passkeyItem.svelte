<script lang="ts">
    import Button from "$comps/button.svelte";
    import * as LogItem from "$comps/logItem/index";
    // "Complex binding patterns require an initialization value" when lang="ts" is not set... OH CMON!
    import Tag from "$comps/tag.svelte";
    import {
        IconCaretDownFilled,
        IconPencilFilled,
        IconTrashXFilled
    } from "@tabler/icons-svelte-runes";

    import type { Passkey } from "#lib/api/model/index.js";
    import { actionDate } from "#lib/common.js";
    import { getAaguidMetadata } from "#lib/passkey/index.js";

    import Symbol from "../symbol.svelte";

    type Props = {
        // id: string;
        // name?: string;
        // tag?: string;
        // when?: string;
        passkey: Passkey;
        onDelete: (id: string) => void;
        onRename: (id: string, current_name: string) => void;
    };

    let { passkey, onDelete, onRename }: Props = $props();

    let when = $derived.by(() => {
        let header = passkey.last_used_at ? "Last used " : "Created ";
        return header + actionDate(passkey.last_used_at ?? passkey.created_at);
    });

    let metadata = $derived.by(() => {
        return getAaguidMetadata(passkey.aaguid);
    });
</script>

<LogItem.Root collapsible collapsible_bg={false}>
    <!-- <div class="icon"> -->
    {#if metadata.icon_dark || metadata.icon_light}
        <picture>
            {#if metadata?.icon_dark}
                <source srcset={metadata.icon_dark} media="(prefers-color-scheme: dark)" />
            {/if}
            {#if metadata?.icon_light}
                <img src={metadata.icon_light} class="auth-icon" alt={metadata.name + " Logo"} />
            {/if}
        </picture>
    {/if}
    <!-- </div> -->
    <LogItem.Header title={passkey.display_name} {when} />
    <LogItem.Actions>
        <Symbol symbol={IconCaretDownFilled} size="medium" />
    </LogItem.Actions>

    <!-- <LogItem.Actions>
        <span>T</span>
    </LogItem.Actions> -->

    {#snippet inner_content()}
        <div class="meow">
            <Button
                onclick={() => onRename(passkey.id, passkey.display_name)}
                shouldFill
                fontSize="small"
                symbol_prefix={{ symbol: IconPencilFilled }}>Rename</Button
            >
            <Button
                onclick={() => onDelete(passkey.id)}
                negative
                shouldFill
                fontSize="small"
                symbol_prefix={{ symbol: IconTrashXFilled }}
            >
                Delete
            </Button>
        </div>
    {/snippet}
</LogItem.Root>

<style lang="scss">
    .meow {
        inline-size: 100%;
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: calc(var(--spacing) * 2);

        @media (max-width: 600px) {
            justify-content: center;
            grid-template-columns: 1fr;
            grid-template-rows: repeat(2, minmax(0, 1fr));
        }
    }

    .auth-icon {
        inline-size: 1.5rem;
        block-size: 1.5rem;
        object-fit: contain;
    }
</style>
