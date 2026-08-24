<script lang="ts">
    import Button from "$comps/button.svelte";
    import { auth } from "$lib/auth/auth.svelte";

    type Props = {
        recoveryCodes?: string[];
    };

    let { recoveryCodes = [] }: Props = $props();

    const downloadRecoveryCodes = $derived.by(() => {
        const text = [
            `${auth.user?.login} - Two-Factor recovery codes`,
            "Each one can be only used once",
            "",
            ...recoveryCodes
        ].join("\n");

        const blob = new Blob([text], { type: "text/plain" });
        return URL.createObjectURL(blob);
    });
</script>

<p>
    Save these back-up codes in a safe place. They can be used when you lose access to your
    authenticator and cannot generate codes. If these are lost, you will loose access to your
    account.
</p>

<ul class="backup-codes">
    {#each recoveryCodes ?? [] as code}
        <li>{code}</li>
    {/each}
</ul>

<Button href={downloadRecoveryCodes} download="meow-auth-recovery-codes.txt">Download</Button>

<style lang="scss">
    .backup-codes {
        display: grid;
        grid-template-columns: auto auto;
        background: var(--color-body);
        border-radius: var(--typical-radius);
        border: 1.5px dashed var(--color-iron-dark);
        font-family: var(--font-mono);
        // max-inline-size: 300px;
        // margin-inline: auto;
        align-items: center;
        justify-items: center;
        list-style-type: none;
        padding-block: calc(var(--spacing) * 2);
        padding-inline: var(--spacing);
        font-weight: 600;

        li {
            inline-size: 100%;
            text-align: center;
        }
    }
</style>
