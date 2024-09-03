<script lang="ts">
	import languages from '$i18n/Languages';
	import { t, getLangName, type Language, setLang} from '$i18n';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';
		
	export let value: Language | string;
    export let set: (newValue: string) => void;

	const LanguageKeys = Object.keys(languages) as Language[];

</script>

<div class="flex items-center">
	<Label text={t("Language")}>{t("Language description")}</Label>

	<Dropdown
		class="flex-grow"
		items= { LanguageKeys }
		selected= { value }
		onSelectedChangeSingle={async (newValue) => {
			value = newValue;
			setLang(newValue);
            set(newValue);
		}}
		getLabel={(name) => getLangName(name)}
	/>
</div>