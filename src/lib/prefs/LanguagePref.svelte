<script lang="ts">
	import languages from '$i18n/Languages';
	import { t, getLangName, type Language, setLang, language, InitLang} from '$i18n';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';
	import { get } from 'svelte/store';
	import BigButton from '$lib/components/BigButton.svelte';
	import { relaunch } from '@tauri-apps/plugin-process';
	
	let value: Language | string = get(language);
    export let set: (newValue: string) => void;

	const LanguageKeys = Object.keys(languages) as Language[];

	async function reboot()
	{
		await relaunch();
	}

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

{#if value != InitLang}
<div class="flex justify-end">
		<BigButton on:click={reboot}>
			<Label text={t("Immediate reboot")}>{t("Immediate reboot description")}</Label>
		</BigButton>
</div>
{/if}