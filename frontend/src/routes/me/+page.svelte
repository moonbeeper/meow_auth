<main id="main" class="grid">
    <div class="settings">
        <section class="settings--panel">
            <header class="settings--header">
                <h2>Profile</h2>
                <p>Its your identify when you sign in to other apps!</p>
            </header>
            <div class="settings--panel-content">
                <div class="flex gap-4 justify-between align-center baba">
                    <div class="avatar aa">?</div>

                    <div class="upload-zone">
                        <p><a href="#main">Upload image</a> or drag and drop</p>
                        <p>PNG, JPEG or WEBP (max. 256x256, 256KiB)</p>
                    </div>
                </div>
                <div class="flex flex-col gap-2 text-start">
                    <label>Name</label>
                    <input class="input" value="Cow69" placeholder="Name" />
                </div>
                <div class="flex flex-col gap-2 text-start">
                    <label>Email address</label>
                    <div class="flex input input--proxy">
                        <input class="input" value="meow@meow.com" placeholder="Email address" />
                        <a href="#main">Meow</a>
                    </div>
                </div>
            </div>
        </section>
        <section class="settings--panel">
            <header class="settings--header">
                <h2>Audit log</h2>
                <p>A little trail of your recent activity on your account</p>
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

    .upload-zone {
        padding-block: calc(var(--spacing) * 4);
        padding-inline: calc(var(--spacing) * 6);
        border-radius: var(--typical-radius);
        border: 1px dashed var(--color-iron-dark);

        justify-content: center;
        display: flex;
        flex-direction: column;

        p {
            font-size: var(--text-small);
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
