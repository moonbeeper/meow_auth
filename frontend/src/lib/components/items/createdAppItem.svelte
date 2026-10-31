<script lang="ts">
    import { goto } from "$app/navigation";
    import * as LogItem from "$comps/logItem/";
    import { IconCaretRightFilled } from "@tabler/icons-svelte-runes";

    import type { ListDataResponseOauthApplicationDataItem } from "#lib/api/model/index.js";
    import { actionDate } from "#lib/common.js";

    import Symbol from "../symbol.svelte";

    type Props = {
        item: ListDataResponseOauthApplicationDataItem;
    };
    let { item }: Props = $props();

    let title = $derived.by(() => {
        return item.name;
    });
    let when = $derived.by(() => {
        return "created " + actionDate(item.created_at);
    });

    let redirect = $derived.by(() => {
        return "/me/my-apps/" + item.id;
    });
</script>

<LogItem.Root asButton onclick={() => goto(redirect)}>
    <LogItem.Header {title} {when} />
    <LogItem.Actions>
        <Symbol symbol={IconCaretRightFilled} size="medium" />
    </LogItem.Actions>
</LogItem.Root>
