<script lang="ts">
	import InputField from "$lib/components/InputField.svelte";
	import { getFont, setFont } from "$lib/theme";
    import Label from "../components/Label.svelte";

    let value = getFont();
</script>

<div class="flex items-center">
	<Label>Font Family</Label>

	<InputField bind:value={value} on:change={() => setFont(value)} />
</div>
