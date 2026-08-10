<script lang="ts">
    import Spinner from "$comps/spinner.svelte";
    import type { FormEventHandler, HTMLInputAttributes } from "svelte/elements";
    import { fade, fly, slide } from "svelte/transition";

    import Input from "./input.svelte";
    // based off bits-ui pin input!
    import { PIN_DIGIT_REGEX, SPACE } from "./regex";

    type PinInputProps = Omit<HTMLInputAttributes, "value"> & Props;
    type Props = {
        value?: string;
        regex?: RegExp;
        maxLength?: number;
        onComplete?: () => void;
        /** A callback that gets a String and spits another String to be parsed by the actual onPaste
         *
         * This is useful when you need to manipulate the pasted string before it gets filtered to remove things
         * like dashes or underscores.
         */
        onPaste?: (s: string) => string;
        loading?: boolean;
    };

    let {
        value = $bindable<string>(""),
        regex = PIN_DIGIT_REGEX,
        maxLength = 6,
        onComplete,
        loading = false,
        onPaste,
        disabled,
        ...rest
    }: PinInputProps = $props();

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
    let space_exp = new RegExp(SPACE);
    let prev_value = $state<string | undefined>(undefined);

    function onkeydown(e: KeyboardEvent) {
        const key = e.key;
        if (KEYS_TO_IGNORE.includes(key)) return;
        if (e.ctrlKey || e.metaKey) return;
        if (key && !reg_exp.test(key)) {
            e.preventDefault();
        }
    }

    // fixes random goofy spaces that can be pasted in (they eat up the max length)
    function onpaste(e: ClipboardEvent) {
        const text = e.clipboardData?.getData("text") ?? "";
        const actual_text = onPaste?.(text) ?? text;

        const filtered = [...actual_text]
            .filter((v) => !space_exp.test(v) && reg_exp.test(v))
            .join("")
            .slice(0, maxLength);
        e.preventDefault();
        value = filtered;
    }

    // fixes that mobile devices can still keep writting past the max length even if set in the input.
    // somehow phones are special, while desktop browsers DO RESPECT IT >:/ (probably my phone is weird idk)
    function oninput(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.value.length > maxLength) {
            input.value = input.value.slice(0, maxLength);
        }
    }

    let dots = $derived.by(() => {
        return "•".repeat(maxLength);
    });

    // nabbed from bits-ui pin input!!!
    $effect(() => {
        if (
            prev_value != undefined &&
            value != prev_value &&
            prev_value.length < maxLength &&
            value.length === maxLength
        ) {
            console.log("submit pin input");
            onComplete?.();
        }

        prev_value = value;
    });

    let isDisabled = $derived.by(() => {
        return loading || disabled;
    });
</script>

<div class="pin-input">
    <Input
        type="text"
        autocomplete="one-time-code"
        required
        autocorrect="off"
        autocapitalize="off"
        autofocus
        maxlength={maxLength}
        placeholder={dots}
        disabled={isDisabled}
        bind:value
        {oninput}
        {onkeydown}
        {onpaste}
        {...rest}
    />
    {#if loading}
        <span class="spinner" transition:fly>
            <Spinner />
        </span>
    {/if}
</div>

<style lang="scss">
    .pin-input {
        position: relative;
        inline-size: 100%;
    }

    .spinner {
        display: flex;
        inset: 0;
        position: absolute;
        align-items: center;
        justify-content: center;
        // flex-shrink: 0;
        // inline-size: var(--button-font-size, 1em);
        // block-size: var(--button-font-size, 1em);
        pointer-events: none;
    }
</style>
