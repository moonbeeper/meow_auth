<script>
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";
</script>

<AuthBox.Root>
    <AuthBox.Header title="Sign up" />
    <form action="meow" class="authbox--form" method="post">
        <input type="text" class="input font-large" placeholder="Your email address" />
        <p class="font-medium">Already have an account? <a href="/signup">Log in</a>!</p>
        <!-- <button class="button button--primary font-medium">Continue</button> -->
        <Button primary fontSize="medium">Continue</Button>
    </form>
</AuthBox.Root>
