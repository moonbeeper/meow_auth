<script lang="ts">
    import * as LogItem from "$comps/logItem/index";
    import type { ListDataResponseAuditLogDataItem } from "$lib/api/model";
    import { actionCategory, actionLabel, intoAuditAction } from "$lib/auditLog";
    import { actionDate } from "$lib/common";
    import { Collapsible } from "bits-ui";
    import type { Snippet } from "svelte";

    type Props = {
        // action: string;
        // was_self: boolean;
        // created_at: string;
        // resource_type?: string;
        // resource_id?: string;
        // actor_ip?: string;
        // actor_location?: string;
        // actor_user_agent?: string;
        // children?: Snippet;
        item: ListDataResponseAuditLogDataItem;
    };

    let { item }: Props = $props();

    let title = $derived.by(() => {
        return actionLabel(intoAuditAction(item.action));
    });
    let tag = $derived.by(() => {
        return actionCategory(intoAuditAction(item.action));
    });
    let who = $derived.by(() => {
        return item.was_self ? "you" : "admin";
    });
    let when = $derived.by(() => {
        return actionDate(item.created_at);
    });
</script>

<LogItem.Root collapsible>
    <LogItem.Header {title} {tag} {who} {when} />
    <!-- <LogItem.Actions>
        <span>V</span>
    </LogItem.Actions> -->

    {#snippet inner_content()}
        <div class="meow">
            <!-- <span>Resource: Application</span> -->
            {#if item.actor_ip}
                <span>{item.actor_ip}</span>
            {/if}
            {#if item.actor_location}
                <span>{item.actor_location}</span>
            {/if}
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

        @media (max-width: 600px) {
            justify-content: center;
        }
    }
</style>
