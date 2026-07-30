<div class="upload-zone">
    <p><a href="#main">Upload image</a> or drag and drop</p>
    <span class="subtitle">PNG, JPEG or WEBP (max. 256x256, 256KiB)</span>
</div>

<style lang="scss">
    .upload-zone {
        padding-block: calc(var(--spacing) * 4);
        padding-inline: calc(var(--spacing) * 6);
        border-radius: var(--typical-radius);
        border: 1px dashed var(--color-iron-dark);

        justify-content: center;
        display: flex;
        flex-direction: column;

        font-size: var(--text-small);
    }
    .subtitle {
        font-size: var(--text-smaller);
        font-weight: 500;
    }
</style>
