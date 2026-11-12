import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
} from "@/components/ui/card";
import { useTranslations } from "@/lib/i18n";

export function ServerErrorPage({ onRetry }: { onRetry: () => void }) {
  const t = useTranslations("errorPages.server");

  return (
    <div className="relative flex min-h-svh items-center justify-center overflow-hidden bg-mesh p-6">
      <div className="absolute inset-0 bg-grid [mask-image:linear-gradient(to_bottom,black,transparent_80%)] opacity-20" />
      <Card className="relative w-full max-w-md gap-4 text-center">
        <CardHeader className="items-center">
          <p className="font-display text-7xl leading-none text-primary/80">
            500
          </p>
          <h1 className="font-display text-3xl leading-tight tracking-tight">
            {t("title")}
          </h1>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button onClick={onRetry}>{t("retry")}</Button>
        </CardContent>
      </Card>
    </div>
  );
}
