<main id="main" class="grid">
    <div class="settings">
        <section class="settings--panel">
            <header class="settings--header">
                <h2>Two-Factor Authentication</h2>
                <p>
                    Add a second layer of protection to your account! When enabled, you'll need a
                    code from your authentictor app to sign in
                </p>
            </header>
            <div class="settings--panel-content">
                <button class="button">Setup two-factor authentication</button>
                <button class="button">Setup two-factor authentication</button>
            </div>
        </section>
        <section class="settings--panel">
            <header class="settings--header">
                <h2>Passkeys</h2>
                <p>Ah, efficient sign in. Modern and more secure!</p>
            </header>
            <div class="settings--panel-content">
                <div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                    </div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                    </div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                    </div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                        <span> MEOW </span>
                        <div>
                            asss
                            <p>kdkasdksa</p>
                            <c>eofkpoafkopasd</c>
                            <a href="#main">MEOW LINK!!1</a>
                        </div>
                    </div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                    </div>
                </div>
            </div>
        </section>
        <section class="settings--panel">
            <header class="settings--header">
                <h2>Active sessions</h2>
                <p>
                    These devices are currently signed in to your account. Revoke any session you
                    don't recognise
                </p>
            </header>
            <div class="settings--panel-content">
                <div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                    </div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                    </div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                    </div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                        <span> MEOW </span>
                        <div>
                            asss
                            <p>kdkasdksa</p>
                            <c>eofkpoafkopasd</c>
                            <a href="#main">MEOW LINK!!1</a>
                        </div>
                    </div>
                    <div class="audit-log">
                        <div class="content">
                            <div class="flex gap-2">
                                <p>Created a new session</p>
                                <p>[tag here]</p>
                            </div>
                            <p>you · 69 years ago</p>
                        </div>
                        <span>V</span>
                    </div>
                </div>
            </div>
        </section>
    </div>
</main>

<style lang="scss">
    .aa {
        position: relative;
        width: 96px;
        height: 96px;
        aspect-ratio: 1;
        /* border: 1px solid var(--color-iron-dark); */

        &::before {
            position: absolute;
            content: "";
            width: 100%;
            height: 100%;
            border-radius: 50%;
            border: 1px solid var(--color-iron-darkest);
        }
    }

    .audit-log {
        display: flex;
        border-bottom: 1px solid var(--color-iron-dark);
        border-inline: 1px solid var(--color-iron-dark);

        padding-block: calc(var(--spacing) * 3);
        padding-inline: calc(var(--spacing) * 5);
        align-items: center;
        gap: calc(var(--spacing) * 4);
        text-align: start;

        &:first-child {
            border-top-left-radius: var(--typical-radius);
            border-top-right-radius: var(--typical-radius);
            border-top: 1px solid var(--color-iron-dark);
        }

        &:last-child {
            border-bottom-left-radius: var(--typical-radius);
            border-bottom-right-radius: var(--typical-radius);
            border-bottom: 1px solid var(--color-iron-dark);
        }

        .content {
            display: flex;
            flex: 1;
            flex-direction: column;
        }
    }

    @media (max-width: 768px) {
        .baba {
            flex-direction: column;
            align-items: center;
        }
    }
</style>
