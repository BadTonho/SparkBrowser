<script lang="ts">
  import { languagePreference, setLanguage, t, type LanguagePreference } from "../../i18n";
</script>
<div class="setting-row" id="setting-language">
  <div class="row-text"><label for="interface-language" class="row-label">{$t("common.language")}</label><span class="row-desc">{$t("common.language_description")}</span></div>
  <select id="interface-language" data-testid="interface-language" class="engine-select" value={$languagePreference} onchange={event => setLanguage(event.currentTarget.value as LanguagePreference)}>
    <option value="system">{$t("common.system")}</option><option value="pt-BR">Português</option><option value="en">English</option>
  </select>
</div>
