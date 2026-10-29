<script lang="ts">
    import Button from "$comps/button.svelte";
    import * as LogItem from "$comps/logItem/index";

    import type { ListDataResponseOauthAuthorizationDataItem } from "#lib/api/model/index.js";
    import { actionDate } from "#lib/common.js";
    import { OauthScope, OauthScopes } from "#lib/oauthScopes.js";

    type Props = {
        item: ListDataResponseOauthAuthorizationDataItem;
        onRevoke: (item: ListDataResponseOauthAuthorizationDataItem) => void;
    };

    let { item, onRevoke }: Props = $props();

    let title = $derived.by(() => {
        return item.name;
    });
    let tag = $derived.by(() => {
        return "External";
    });
    let when = $derived.by(() => {
        return "last used " + actionDate(item.last_used_at ?? item.created_at);
    });

    let scopes = $derived.by(() => {
        return new OauthScopes(item.scopes).remove(OauthScope.OpenId).toStrings();
    });
</script>

<LogItem.Root collapsible>
    <LogItem.Header {title} {tag} {when} />
    {#snippet inner_content()}
        <div class="meow">
            <div class="access">
                <p class="can">Can read:</p>
                <ul class="scopes">
                    {#each scopes as scope}
                        <li>{scope}</li>
                    {/each}
                </ul>
            </div>
            <Button negative fontSize="small" onclick={() => onRevoke(item)}>Revoke access</Button>
        </div>
    {/snippet}
</LogItem.Root>

<style lang="scss">
    .meow {
        inline-size: 100%;
        display: flex;
        flex-direction: column;
        align-items: start;
        gap: calc(var(--spacing) * 2);
        font-size: var(--text-small);
    }

    .access {
        text-align: start;
        inline-size: 100%;
        display: grid;
        grid-template-columns: auto 1fr;
        gap: calc(var(--spacing) * 2);

        @media (max-width: 600px) {
            justify-content: center;
            grid-template-columns: 1fr;
            grid-template-rows: auto 1fr;
        }
    }

    .scopes {
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * 2);
        margin: 0;
        margin-left: calc(var(--spacing) * 6);
        margin-bottom: calc(var(--spacing) * 2);
        padding-inline-start: 0;
        align-items: start;
    }
</style>
