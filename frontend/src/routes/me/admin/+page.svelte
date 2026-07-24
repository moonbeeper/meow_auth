<main id="main" class="grid">
    <div class="settings">
        <section class="settings--panel">
            <header class="settings--header">
                <h2>oh ah god</h2>
                <p>meow</p>
            </header>
            <div class="settings--panel-content">
                <button class="button">oh god</button>
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
