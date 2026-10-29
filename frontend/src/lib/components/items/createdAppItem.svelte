<script lang="ts">
    import { goto } from "$app/navigation";
    import * as LogItem from "$comps/logItem/index";
    import type { ListDataResponseOauthApplicationDataItem } from "$lib/api/model";
    import { actionDate } from "$lib/common";

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
        <span>></span>
    </LogItem.Actions>
</LogItem.Root>
