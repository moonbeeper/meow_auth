<script lang="ts">
    import type { HTMLInputAttributes } from "svelte/elements";

    let { value = $bindable(), ...rest }: HTMLInputAttributes = $props();
</script>

<label>
    <input class="input" {value} {...rest} />
</label>
