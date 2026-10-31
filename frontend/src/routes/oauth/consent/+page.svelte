<script lang="ts">
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";
    import { onMount } from "svelte";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import {
        createAvailableScopesOauthGlobal,
        finishAuthorization,
        getFinishAuthorizationUrl
    } from "#lib/api/index.js";

    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();

    const availableScopesQuery = createAvailableScopesOauthGlobal();
    let availableScopes = $derived.by(() => {
        const res = availableScopesQuery.data;
        if (!res || res.status !== 200) return [];
        return res.data;
    });

    let scopes = $derived.by(() => {
        let itemScopes = data.consent.scopes.split(" ");
        return availableScopes
            .filter((scope) => itemScopes.includes(scope.identifier))
            .filter((scope) => scope.identifier != "openid")
            .map((scope) => scope);
    });
    let submit_disabled = $state(true);
    let submit_loading = $state(false);

    async function handleAuthorize(yes: boolean = false) {
        submit_loading = true;
        const res = await finishAuthorization({ client_id: data.consent.id, consent: yes });

        if (!isOk(res)) {
            console.error("i will cry");
            submit_loading = false;
            return;
        }
        console.log("finalized authorization");
        submit_loading = false;
    }

    onMount(() => {
        setTimeout(() => {
            submit_disabled = false;
        }, 4000);
    });
</script>

<AuthBox.Root>
    <div class="who-to-who">
        <div class="figure--image">A</div>
        ->
        <div class="figure--image figure--image--user">A</div>
    </div>
    <AuthBox.Header title="Authorize {data.consent.name}" subtitle="access your account" />
    <div class="list-container">
        <p>It will be able to read:</p>
        <ul class="list">
            {#each scopes as scope}
                <li>{scope.description}</li>
            {/each}
        </ul>
    </div>
    <form method="post" action={getFinishAuthorizationUrl()} class="actions">
        <input type="hidden" name="client_id" value={data.consent.id} />
        <Button
            fontSize="medium"
            shouldFill
            onclick={() => handleAuthorize()}
            disabled={submit_loading}>Cancel</Button
        >
        <Button
            primary
            fontSize="medium"
            shouldFill
            disabled={submit_disabled || submit_loading}
            type="submit"
            name="consent"
            value="true">Authorize</Button
        >
    </form>
</AuthBox.Root>

<style lang="scss">
    .actions {
        margin-block-start: calc(var(--spacing) * 4);
        display: flex;
        inline-size: 100%;
        gap: calc(var(--spacing) * 4);
    }

    .list-container {
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * 1);
        // justify-content: center;
        align-items: center;
    }

    .list-container p {
        @media (prefers-color-scheme: dark) {
            color: var(--color-accent-lighter);
        }

        font-weight: 600;
    }

    .list {
        margin: 0 auto;
        padding: 0;
        text-align: center;
        list-style: none;
    }

    .list li {
        position: relative;
        width: fit-content;
        margin: 0 auto;

        &::before {
            content: "•";
            position: absolute;
            right: 100%;
            margin-right: 0.5em;
        }
    }

    .who-to-who {
        min-inline-size: 100%;
        display: flex;
        justify-content: center;
        align-items: center;

        gap: calc(var(--spacing) * 6);
    }

    .figure--image {
        position: relative;
        // width: 96px;
        // height: 96px;
        inline-size: 4rem;
        block-size: 4rem;
        flex-shrink: 0;
        // aspect-ratio: 1;
        border-radius: var(--img-radius, var(--typical-radius));
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 600;
        font-size: var(--text-small);
        // overflow: hidden;
        border: 1px solid var(--color-accent-light);

        // subtle as HECK but i like it
        box-shadow: rgba(0, 0, 0, 0.05) 0px 6px 24px 0px;
    }

    .figure--image--user {
        --img-radius: 50%;
    }
</style>
