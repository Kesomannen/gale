<script lang="ts">
	import { t, getLangName, type Language, language, InitLang, LanguageKeys} from '$i18n';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';
	import { get } from 'svelte/store';
	import BigButton from '$lib/components/BigButton.svelte';
	import { relaunch } from '@tauri-apps/plugin-process';
	import Tooltip from '$lib/components/Tooltip.svelte';
	
	let value: Language | string = language;
    export let set: (newValue: string) => void;
</script>

<div class="flex items-center">
	<Label text={t("Language")}>{t("Language description")}</Label>

	<Dropdown
		class="flex-grow"
		items= { LanguageKeys }
		selected= { value }
		onSelectedChangeSingle={async (newValue) => {
			value = newValue;
            set(newValue);
			location.reload();
		}}
		getLabel={(name) => getLangName(name)}
	/>
</div>

{#if value != InitLang}
<div class="flex justify-end">
		<BigButton on:click={location.reload}>
			<Tooltip text={t("Immediate reload description")}>{t("Immediate reload")}</Tooltip>
		</BigButton>
</div>
{/if}