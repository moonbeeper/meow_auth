<script lang="ts">
    import { version } from "$app/env";
    import Header from "$comps/header.svelte";
    import favicon from "$lib/assets/favicon.svg";

    import "../styles/_global.scss";
    import "@fontsource-variable/inter/wght.css";

    let { children } = $props();
    let build_date = new Date(Number(version)).toTimeString();
    // remember: cannot use crypto stuff [how would have known that, i didnt :(]
</script>

<svelte:head>
    <link rel="icon" href={favicon} />
</svelte:head>

<Header />
{@render children()}
<footer class="footer">
    <div class="footer--content">
        Running on {__COMMIT_HASH__} built at {build_date}
    </div>
</footer>

<style lang="scss">
</style>
