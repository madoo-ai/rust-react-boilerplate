import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
} from "@/components/ui/card";
import { useTranslations } from "@/lib/i18n";

export function NotFoundPage() {
  const t = useTranslations("errorPages.notFound");
  const tCommon = useTranslations("common");

  return (
    <div className="relative flex min-h-svh items-center justify-center overflow-hidden bg-mesh p-6">
      <div className="absolute inset-0 bg-grid [mask-image:linear-gradient(to_bottom,black,transparent_80%)] opacity-20" />
      <Card className="relative w-full max-w-md gap-4 text-center">
        <CardHeader className="items-center">
          <p className="font-display text-7xl leading-none text-primary/80">
            404
          </p>
          <h1 className="font-display text-3xl leading-tight tracking-tight">
            {t("title")}
          </h1>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button asChild>
            <Link to="/">{tCommon("backToHome")}</Link>
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
