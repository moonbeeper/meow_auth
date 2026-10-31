<script lang="ts">
    import * as LogItem from "$comps/logItem/index";
    import { IconCaretDownFilled, IconTrashXFilled } from "@tabler/icons-svelte-runes";
    // "Complex binding patterns require an initialization value" when lang="ts" is not set... OH CMON!
    import { UAParser } from "ua-parser-js";

    import type { ListDataResponseSessionDataItem } from "#lib/api/model/listDataResponseSessionDataItem.js";
    import { actionDate } from "#lib/common.js";

    import Button from "../button.svelte";
    import Symbol from "../symbol.svelte";

    type Props = {
        item: ListDataResponseSessionDataItem;
        onDelete: (id: string) => void;
    };

    let { item, onDelete }: Props = $props();

    let ua_parser = $derived(new UAParser($state.snapshot(item.user_agent))); // oh you can just do $derived lmao

    let title = $derived.by(() => {
        return (
            (ua_parser.getBrowser().name || "Unknown") +
            " (" +
            (ua_parser.getOS().name || "Unknown OS") +
            ")"
        );
    });

    let tag = $derived.by(() => {
        let device = ua_parser.getDevice().type || "desktop";
        return String(device).charAt(0).toUpperCase() + String(device).slice(1);
    });

    let when = $derived.by(() => {
        return "Created " + actionDate(item.created_at);
    });

    let expires = $derived.by(() => {
        return actionDate(item.expires_at);
    });
</script>

<LogItem.Root collapsible>
    <LogItem.Header {title} {tag} {when} />
    <LogItem.Actions>
        <Symbol symbol={IconCaretDownFilled} size="medium" />
    </LogItem.Actions>

    {#snippet inner_content()}
        <div class="meow">
            <span><b>Expires {expires}</b></span>

            <Button
                negative
                shouldFill
                fontSize="small"
                onclick={() => onDelete(item.id)}
                symbol_prefix={{ symbol: IconTrashXFilled }}
            >
                Revoke
            </Button>
        </div>
    {/snippet}
</LogItem.Root>

<style lang="scss">
    .meow {
        display: flex;
        flex-direction: row;
        gap: calc(var(--spacing) * 2);
        font-size: var(--text-smaller);
        flex: 1;
        flex-wrap: wrap;
        align-items: center;
    }
</style>
