<script lang="ts">
    import * as LogItem from "$comps/logItem/index";
    import { Collapsible } from "bits-ui";
    import type { Snippet } from "svelte";

    let {
        title = "space birb",
        tag = "hatch",
        who = "me",
        when = "right now",
        children
    }: {
        title?: string;
        tag?: string;
        who?: string;
        when?: string;
        children?: Snippet;
    } = $props();
</script>

<LogItem.Root>
    <LogItem.Header {title} {tag} {who} {when} />
    <!-- <LogItem.Actions>
        <span>V</span>
    </LogItem.Actions> -->
</LogItem.Root>
