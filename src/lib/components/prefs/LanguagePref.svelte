<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { changeLanguage, languageTitle, locales } from '$lib/i18n';

	const items = locales.map((option) => ({
		label: languageTitle[option](),
		value: option,
	}));

	type Props = {
		value: string;
		set: (newValue: string) => void;
	};

	let { value = $bindable(), set }: Props = $props();
</script>

<div class="flex items-center">
	<Label>Language</Label>

	<Select
		type="single"
		triggerClass="grow"
		{items}
        value={value}
		onValueChange={(newValue) => {
            changeLanguage(newValue);
			set(newValue);
		}}
	/>
</div>