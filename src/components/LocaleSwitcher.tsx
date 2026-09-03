import { StyleSheet, Text, TouchableOpacity, View } from "react-native";
import { SUPPORTED_LOCALES, type Locale } from "../i18n";

interface LocaleSwitcherProps {
  locale: Locale;
  onChange: (locale: Locale) => void;
}

export function LocaleSwitcher({ locale, onChange }: LocaleSwitcherProps) {
  return (
    <View style={styles.localeSwitch}>
      {SUPPORTED_LOCALES.map((loc) => (
        <TouchableOpacity
          key={loc}
          style={[styles.localeBtn, locale === loc && styles.localeBtnActive]}
          onPress={() => onChange(loc)}
        >
          <Text
            style={[
              styles.localeBtnText,
              locale === loc && styles.localeBtnTextActive,
            ]}
          >
            {loc.toUpperCase()}
          </Text>
        </TouchableOpacity>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  localeSwitch: {
    flexDirection: "row",
    gap: 6,
    marginTop: 10,
  },
  localeBtn: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: "#cbd5e1",
    backgroundColor: "#f1f5f9",
  },
  localeBtnActive: {
    borderColor: "#4f46e5",
    backgroundColor: "#eef2ff",
  },
  localeBtnText: {
    fontSize: 10,
    fontWeight: "800",
    color: "#64748b",
  },
  localeBtnTextActive: {
    color: "#3730a3",
  },
});
