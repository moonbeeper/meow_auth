<script lang="ts">
    import type { HTMLInputAttributes } from "svelte/elements";

    // based off bits-ui pin input!
    import { PIN_DIGIT_REGEX } from "./regex";

    let {
        value = $bindable(),
        regex = PIN_DIGIT_REGEX,
        max_length = 6,
        ...rest
    }: HTMLInputAttributes & {
        regex?: string;
        max_length?: number;
    } = $props();

    const KEYS_TO_IGNORE = [
        "Backspace",
        "Delete",
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
        "Home",
        "End",
        "Escape",
        "Enter",
        "Tab",
        "Shift",
        "Control",
        "Meta"
    ];

    let reg_exp = $derived.by(() => {
        return new RegExp(regex);
    });

    function onkeydown(e: KeyboardEvent) {
        const key = e.key;
        if (KEYS_TO_IGNORE.includes(key)) return;
        if (e.ctrlKey || e.metaKey) return;
        if (key && !reg_exp.test(key)) {
            e.preventDefault();
        }
    }

    let dots = $derived.by(() => {
        return "•".repeat(max_length);
    });
</script>

<input
    class="input"
    type="text"
    autocomplete="one-time-code"
    required
    autocorrect="off"
    autocapitalize="off"
    maxlength={max_length}
    placeholder={dots}
    bind:value
    {onkeydown}
    {...rest}
/>
