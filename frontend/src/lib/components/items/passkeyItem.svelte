<script lang="ts">
    import * as LogItem from "$comps/logItem/index";
    // "Complex binding patterns require an initialization value" when lang="ts" is not set... OH CMON!
    import Tag from "$comps/tag.svelte";

    let {
        title = "my favorite passkey",
        tag = "disabled",
        when = "Last used 2 days ago"
    }: { title?: string; tag?: string; when?: string } = $props();
</script>

<LogItem.Root>
    <LogItem.Header {title} {tag} {when} />
    <LogItem.Actions>
        <span>T</span>
    </LogItem.Actions>
</LogItem.Root>
