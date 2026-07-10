<script lang="ts">
    import { browser } from "$app/env";

    const greetings = [
        "Meow!",
        "Psst, over here!",
        "Another wanderer...",
        "You rang?",
        "Greetings, traveler",
        "Oh, it's you"
    ];

    let greeting = $derived.by(() => {
        if (browser) {
            return greetings[Math.floor(Math.random() * greetings.length)];
        }
    });
</script>

<main id="main" class="grid justify-center margin-auto">
    <div class="flex flex-col align-center gap-2">
        <!-- id like to have this random, like we did with the email greetings -->
        <h1 class="font-x-large margin-block-end">{greeting}</h1>
        <form action="meow" class="gap-3 flex flex-col align-center" method="post">
            <input type="text" class="input font-large" placeholder="Your email address" />
            <p class="font-medium">New around this auth realm? <a href="/signup">Sign up</a>!</p>
            <button class="button button--primary font-medium"><span>Continue</span></button>
        </form>
    </div>
</main>
