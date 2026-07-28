<script lang="ts">
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";

    const greetings = [
        "Meow!",
        "Psst, over here!",
        "Another wanderer...",
        "You rang?",
        "Greetings, traveler",
        "Oh, it's you"
    ];

    let greeting = $derived.by(() => {
        return greetings[Math.floor(Math.random() * greetings.length)];
    });
</script>

<AuthBox.Root>
    <AuthBox.Header title={greeting} />
    <form action="meow" class="authbox--form" method="post">
        <input type="text" class="input font-large" placeholder="Your email address" />
        <p class="font-medium">New around this auth realm? <a href="/signup">Sign up</a>!</p>
        <!-- <button class="button button--primary font-medium">Continue</button> -->
        <Button primary fontSize="medium">Continue</Button>
    </form>
</AuthBox.Root>
