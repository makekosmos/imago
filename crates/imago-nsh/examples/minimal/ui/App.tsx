import { mount, signal, Text, View } from "nsh";
import { Sidebar, Titlebar } from "../../../ui";

const colors = { background: "#0d0c10", foreground: "#e7e5ea", muted: "#8d8a94", border: "#27262a", accent: "#af94f9", card: "#131216" };
const route = signal("home");

function App() {
  const navigation = [{ route: "home", label: "Главная", icon: "inbox" as const }];
  return <View class="h-full w-full flex-col" style={{ backgroundColor: colors.background }}>
    <Titlebar title="Пример" icon="about" colors={colors} onToggleSidebar={() => {}} onAppearance={() => {}} />
    <View class="flex-1 flex-row">
      <Sidebar visible current={route()} navigation={navigation} otherPages={[]} projects={[]} moreOpen={false} colors={colors} onNavigate={route.set} onProject={() => {}} onToggleMore={() => {}} />
      <View class="flex-1 items-center justify-center"><Text style={{ color: colors.foreground }}>Одна страница приложения</Text></View>
    </View>
  </View>;
}

mount(App);
