<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { primaryColorSetting, accentColorSetting } from '$lib/state/theme.svelte';
	import ColorPref from './ColorPref.svelte';
</script>

<ColorPref
	label={m.colorPref_title_accent()}
	setting={primaryColorSetting}
	initialCustomColor="#6b7280">{m.prefs_appearance_color_primary_content()}</ColorPref
>
<ColorPref
	label={m.colorPref_title_primary()}
	setting={accentColorSetting}
	initialCustomColor="#16a34a">{m.prefs_appearance_color_accent_content()}</ColorPref
>
