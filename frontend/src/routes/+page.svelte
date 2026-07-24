<script lang="ts">
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

<main id="main" class="grid justify-center margin-auto">
    <div class="authbox">
        <header class="authbox--header">
            <h1 class="authbox--title">{greeting}</h1>
        </header>
        <form action="meow" class="authbox--form" method="post">
            <input type="text" class="input font-large" placeholder="Your email address" />
            <p class="font-medium">New around this auth realm? <a href="/signup">Sign up</a>!</p>
            <button class="button button--primary font-medium">Continue</button>
        </form>
    </div>
</main>
