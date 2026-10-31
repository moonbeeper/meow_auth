<script lang="ts">
    import * as LogItem from "$comps/logItem/index";
    import { IconCaretDownFilled, IconMapPinFilled, IconWorld } from "@tabler/icons-svelte-runes";

    import type { ListDataResponseAuditLogDataItem } from "#lib/api/model/index.js";
    import {
        actionCategory,
        actionLabel,
        auditFieldLabel,
        displayValue,
        intoAuditAction
    } from "#lib/auditLog.js";
    import { actionDate } from "#lib/common.js";

    import Symbol from "../symbol.svelte";

    type Props = {
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
        switch (item.actor_type) {
            case "admin":
                return item.actor_login || "Admin";
            case "user_self":
                return item.actor_login || "Yourself";
            case "system":
                return "System";
            default:
                return "Unknown";
        }
    });
    let when = $derived.by(() => {
        return actionDate(item.created_at);
    });

    let changes = $derived(item.changes);
    let canExpand = $derived(changes.length > 0 || !!item.actor_ip || !!item.actor_location);
</script>

<LogItem.Root collapsible={canExpand}>
    <LogItem.Header {title} {tag} {who} {when} />
    {#if canExpand}
        <LogItem.Actions>
            <Symbol symbol={IconCaretDownFilled} size="medium" />
        </LogItem.Actions>
    {/if}

    {#snippet inner_content()}
        <div class="meow">
            {#if changes.length > 0}
                <ul class="changes">
                    {#each changes as change}
                        <li class="change">
                            <span class="field">{auditFieldLabel(change.field)}</span>

                            <span class="diff">
                                {#if change.before != null}
                                    <span class="removed">{displayValue(change.before)}</span>
                                {/if}
                                {#if change.before != null && change.after != null}
                                    <span>-></span>
                                {/if}
                                {#if change.after !== null}
                                    <span class="added">{displayValue(change.after)}</span>
                                {/if}
                            </span>
                        </li>
                    {/each}
                </ul>
            {/if}
            <!-- <span>Resource: Application</span> -->
            {#if item.actor_ip}
                <span class="info"><Symbol symbol={IconWorld} />{item.actor_ip}</span>
            {/if}
            {#if item.actor_location}
                <span class="info"><Symbol symbol={IconMapPinFilled} />{item.actor_location}</span>
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

    .info {
        display: inline-flex;
        align-items: center;
        gap: calc(var(--spacing) * 1);
    }

    .changes {
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * 1);
        margin: 0;
        padding: 0;
        list-style: none;
        inline-size: 100%;
    }

    .change {
        display: flex;
        flex-wrap: wrap;
        gap: calc(var(--spacing) * 1);
        align-items: baseline;
    }

    .field {
        font-weight: 600;
    }

    .diff {
        display: inline-flex;
        flex-wrap: wrap;
        gap: calc(var(--spacing) * 1);
        align-items: start;
    }

    .removed {
        color: var(--color-coral-medium);
        text-decoration: line-through;
        @media (prefers-color-scheme: dark) {
            color: var(--color-coral-light);
        }
    }

    .added {
        color: var(--color-emerald-medium);
        @media (prefers-color-scheme: dark) {
            color: var(--color-emerald-light);
        }
    }
</style>
