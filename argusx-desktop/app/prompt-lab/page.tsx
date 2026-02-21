"use client";

import { CheckCircle, XCircle, Folder } from "lucide-react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

const stats = [
  { label: "Checklist Items", value: "2", icon: CheckCircle },
  { label: "Golden Sets", value: "1", icon: Folder },
  { label: "Passed", value: "1", icon: CheckCircle },
  { label: "Failed", value: "0", icon: XCircle },
];

export default function PromptLabDashboard() {
  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">PromptLab Dashboard</h1>
      <div className="grid gap-4 md:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.label}>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">{stat.label}</CardTitle>
              <stat.icon className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold tabular-nums">{stat.value}</div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
