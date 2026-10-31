<script lang="ts">
    import Button from "$comps/button.svelte";
    import * as LogItem from "$comps/logItem/index";
    import { IconCaretDownFilled, IconTrashXFilled } from "@tabler/icons-svelte-runes";

    import { createAvailableScopesOauthGlobal } from "#lib/api/index.js";
    import type { ListDataResponseOauthAuthorizationDataItem } from "#lib/api/model/index.js";
    import { actionDate } from "#lib/common.js";

    import Symbol from "../symbol.svelte";

    type Props = {
        item: ListDataResponseOauthAuthorizationDataItem;
        onRevoke: (item: ListDataResponseOauthAuthorizationDataItem) => void;
    };

    let { item, onRevoke }: Props = $props();

    let title = $derived.by(() => {
        return item.name;
    });
    let tag = $derived.by(() => {
        return item.client_type == "first_party" ? "First party" : "Third party";
    });
    let when = $derived.by(() => {
        return "last used " + actionDate(item.last_used_at ?? item.created_at);
    });

    const availableScopesQuery = createAvailableScopesOauthGlobal();
    let availableScopes = $derived.by(() => {
        const res = availableScopesQuery.data;
        if (!res || res.status !== 200) return [];
        return res.data;
    });

    let scopes = $derived.by(() => {
        let itemScopes = item.scopes.split(" ");
        return availableScopes
            .filter((scope) => itemScopes.includes(scope.identifier))
            .filter((scope) => scope.identifier != "openid")
            .map((scope) => scope);
    });
</script>

<LogItem.Root collapsible>
    <LogItem.Header {title} {tag} {when} />
    <LogItem.Actions>
        <Symbol symbol={IconCaretDownFilled} size="medium" />
    </LogItem.Actions>
    {#snippet inner_content()}
        <div class="meow">
            <div class="access">
                <p class="can">Can read:</p>
                <ul class="scopes">
                    {#each scopes as scope}
                        <li>{scope.description}</li>
                    {/each}
                </ul>
            </div>
            <Button
                negative
                fontSize="small"
                onclick={() => onRevoke(item)}
                symbol_prefix={{ symbol: IconTrashXFilled }}>Revoke access</Button
            >
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
