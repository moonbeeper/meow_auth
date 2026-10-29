<script lang="ts">
    import { page } from "$app/state";
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";
</script>

<AuthBox.Root>
    <AuthBox.Header title="Authorization request expired" subtitle={page.error?.message} />
    <Button href="/" primary fontSize="medium">Back to safety</Button>
</AuthBox.Root>
