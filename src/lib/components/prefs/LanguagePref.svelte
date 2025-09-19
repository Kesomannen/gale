<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { setLanguage, languageTitle } from '$lib/i18n';
	import { m } from '$lib/paraglide/messages';
	import { locales } from '$lib/paraglide/runtime';

	const items = locales.map((option) => ({
		label: languageTitle[option],
		value: option
	}));

	type Props = {
		value: string;
		set: (newValue: string) => void;
	};

	let { value = $bindable(), set }: Props = $props();
</script>

<div class="flex items-center">
	<Label>{m.languagePref_title()}</Label>

	<Select
		type="single"
		triggerClass="grow"
		{items}
		{value}
		onValueChange={(newValue) => {
			setLanguage(newValue);
			set(newValue);
		}}
	/>
</div>
