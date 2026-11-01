<script lang="ts">
    import * as LogItem from "$comps/logItem/index";
    import {
        IconCaretDownFilled,
        IconCaretRightFilled,
        IconCircleArrowLeftFilled,
        IconTrashXFilled
    } from "@tabler/icons-svelte-runes";
    // "Complex binding patterns require an initialization value" when lang="ts" is not set... OH CMON!

    import type { ListDataResponseUserDataItem } from "#lib/api/model/listDataResponseUserDataItem.js";
    import { actionDate } from "#lib/common.js";

    import Button from "../button.svelte";
    import Symbol from "../symbol.svelte";

    type Props = {
        item: ListDataResponseUserDataItem;
    };

    let { item }: Props = $props();

    let title = $derived.by(() => {
        return item.login;
    });

    let when = $derived.by(() => {
        return "Created " + actionDate(item.created_at);
    });
</script>

<LogItem.Root asLink href={`/me/admin/user/${item.id}`}>
    <LogItem.Header {title} {when} />
    <LogItem.Actions>
        <Symbol symbol={IconCaretRightFilled} size="medium" />
    </LogItem.Actions>

    {#snippet inner_content()}
        <div class="meow">
            <Button
                negative
                shouldFill
                fontSize="small"
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
