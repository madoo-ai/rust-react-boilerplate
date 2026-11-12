import { SecuritySettings } from "@/components/settings/security-settings";
import { useSession } from "@/lib/auth";

export function SecuritySettingsPage() {
  const session = useSession();

  if (!session) return null;

  return <SecuritySettings twoFactorEnabled={session.user.twoFactorEnabled} />;
}
