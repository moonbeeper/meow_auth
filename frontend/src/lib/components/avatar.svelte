<script lang="ts">
    import { invalidateAll } from "$app/navigation";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import { logout } from "$lib/api/user/user";
    import { auth } from "$lib/auth/auth.svelte";
    import { Avatar } from "bits-ui";
    import { preventDefault } from "svelte/legacy";

    import * as Dropdown from "./dropdown";

    type Props = {
        small?: boolean;
        interactive?: boolean;
    };

    let { small = false, interactive = false }: Props = $props();

    let loggingOut = $state(false);

    async function handleLogout(e: Event) {
        if (loggingOut) {
            console.warn("already loggin out, ignoring");
            e.preventDefault();
            return;
        }
        console.log("logging out");
        loggingOut = true;
        try {
            const res = await logout();
            if (!isOk(res)) {
                console.error("failed to log out");
                return;
            }
            await invalidateAll();
            console.log("logged out");
        } finally {
            loggingOut = false;
        }
    }
</script>

<!-- SVELTE USES CLSX NOW?!?!?!?!??!?? -->
{#snippet avatar()}
    <div class="container">
        <Avatar.Root class={["avatar", { small }]}>
            <div class="inner">
                <Avatar.Image src="https://github.com/moonbeeper.png" alt="Moonbeeper's avatar" />
                <Avatar.Fallback>HB</Avatar.Fallback>
            </div>
        </Avatar.Root>
    </div>
{/snippet}

{#if interactive}
    <Dropdown.Root>
        {#snippet trigger({ props })}
            <button class="decoration" {...props}>
                {@render avatar()}
            </button>
        {/snippet}
        <Dropdown.Text>
            <p class="user-display">poopy pants</p>
        </Dropdown.Text>
        <Dropdown.Item onSelect={handleLogout}>Log out</Dropdown.Item>
    </Dropdown.Root>
{:else}
    {@render avatar()}
{/if}

<style lang="scss">
    .container {
        // position: relative;
        // inline-size: fit-content;
        // flex-shrink: 0;
        display: contents;
        user-select: none;
    }

    .user-display {
        font-size: var(--text-small);
        font-weight: 500;
        color: var(--color-iron-dark);
    }

    .container :global(.avatar) {
        position: relative;
        // width: 96px;
        // height: 96px;
        inline-size: 6rem;
        block-size: 6rem;
        flex-shrink: 0;
        // aspect-ratio: 1;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 600;
        font-size: var(--text-small);
        // overflow: hidden;
        border: 1px solid var(--color-accent-light);
        outline: 1px solid var(--color-iron-inverted);
        outline-offset: -3px;

        // subtle as HECK but i like it
        box-shadow: rgba(0, 0, 0, 0.05) 0px 6px 24px 0px;
        // &::before {
        //     position: absolute;
        //     content: "";
        //     // width: 100%;
        //     // height: 100%;
        //     inset: 1px;
        //     border-radius: 50%;
        //     border: 1px solid var(--color-accent-light);
        // }
        //
    }

    .container :global(.avatar.small) {
        max-inline-size: 2.25rem;
        max-block-size: 2.25rem;
    }

    .inner {
        display: flex;
        width: 100%;
        height: 100%;
        overflow: hidden;
        justify-content: center;
        align-items: center;
        border-radius: 50%;
    }

    .decoration {
        --focus-outline-offset: 3px;
        all: unset;
        box-sizing: border-box;
        cursor: pointer;
        display: flex;
        justify-content: center;
        align-items: center;
        border-radius: 50%;
        outline: var(--typical-outline-size) solid transparent;
        outline-offset: var(--focus-outline-offset, 1px);

        &:focus-visible {
            outline-color: var(--focus-outline-color, var(--color-accent-light));
        }
    }
</style>
