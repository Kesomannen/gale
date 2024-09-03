<script lang="ts">
	import { t, getLangName, type Language, language } from '$i18n';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';
		
	export let value: Language | string;
    export let set: (newValue: string) => void;

	let items = Object.keys(language) as Language[];

</script>

<div class="flex items-center">
	<Label text={t("Language")}>{t("Language description")}</Label>

	<Dropdown
		class="flex-grow"
		items= { items }
		selected= { value }
		onSelectedChangeSingle={async (newValue) => {
            set(newValue);
		}}
		getLabel={(name) => getLangName(name)}
	/>
</div>