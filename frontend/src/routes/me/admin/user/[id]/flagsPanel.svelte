<script lang="ts">
    import { refreshAll } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import { IconDeviceFloppyFilled } from "@tabler/icons-svelte-runes";
    import { untrack } from "svelte";
    import { slide } from "svelte/transition";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import { createAdminEditUser } from "#lib/api/index.js";
    import type { User } from "#lib/api/model/user.js";
    import { UserFlag, UserFlags } from "#lib/auth/userFlags.js";
    import CheckboxBox from "#lib/components/form/checkbox_box.svelte";

    type Props = {
        user: User;
        onSaved: () => void;
    };

    let { user }: Props = $props();
    const currentFlags = new UserFlags(untrack(() => user.flags));

    type FlagItem = {
        flag: UserFlag;
        label: string;
        checked: boolean;
    };

    const flags = $state<FlagItem[]>(
        new UserFlags().allSet().enumArray.map((flag) => ({
            flag,
            label: new UserFlags(flag).toStrings()[0] ?? "Unknown",
            checked: currentFlags.has(flag)
        }))
    );

    const bits = $derived.by(() => {
        return flags.reduce((bits, flag) => (flag.checked ? bits | flag.flag : bits), 0);
    });

    const dirty = $derived.by(() => {
        return bits != user.flags;
    });

    let saving = $state(false);
    let error = $state<string | null>(null);
    const updateFlags = createAdminEditUser();

    async function save() {
        if (saving || !dirty) return;

        saving = true;
        try {
            const res = await updateFlags.mutateAsync({ id: user.id, data: { flags: bits } });
            if (!isOk(res)) {
                console.error("failed updating user flags: ", res.data.message);
                error = res.data.message ?? "Failed to update user flags :(";
            }
            await refreshAll();
        } catch (err) {
            console.error("failed updating user flags");
            error = "Failed to update user flags :(";
        } finally {
            saving = false;
        }
    }
</script>

<SettingsPanel title="Flags" description="Used to control user access and permissions">
    {#if error}
        <p class="hint">{error}</p>
    {/if}
    {#each flags as flag (flag.flag)}
        <CheckboxBox size="large" bind:checked={flag.checked}>
            <span class="checkbox-box--title">{flag.label}</span>
        </CheckboxBox>
    {/each}

    {#if dirty}
        <div transition:slide>
            <Button
                primary
                disabled={saving}
                loading={saving}
                shouldFill
                symbol_prefix={{ symbol: IconDeviceFloppyFilled }}
                onclick={() => save()}
            >
                Save changes
            </Button>
        </div>
    {/if}
</SettingsPanel>

<style lang="scss">
    .checkbox-box--title {
        font-size: var(--text-small);
        font-weight: 500;
    }

    .hint {
        color: var(--color-coral-medium);
        font-size: var(--text-small);
        font-weight: 500;
        @media (prefers-color-scheme: dark) {
            color: var(--color-coral-light);
        }
    }
</style>
