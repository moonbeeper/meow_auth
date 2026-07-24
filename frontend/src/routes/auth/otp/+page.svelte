<script lang="ts">
    import Pin from "$comps/form/pin.svelte";
    import { PIN_DIGIT_AND_CHAR } from "$comps/form/regex";
</script>

<main id="main" class="grid justify-center margin-auto">
    <div class="flex flex-col align-center gap-1">
        <header>
            <h1 class="font-x-large">Check your email!</h1>
            <p class="font-medium margin-block-end">
                There should be a verification code that you input below:
            </p>
        </header>
        <form action="meow" class="gap-3 flex flex-col align-center" method="post">
            <Pin regex={PIN_DIGIT_AND_CHAR} />
        </form>
    </div>
</main>
