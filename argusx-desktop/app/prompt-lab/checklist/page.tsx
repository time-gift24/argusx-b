"use client";

import { useState, useEffect } from "react";
import { Plus, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import {
  listChecklistItems,
  deleteChecklistItem,
  type ChecklistItem,
} from "@/lib/api/prompt-lab";
import { mockInvoke } from "@/lib/mocks/prompt-lab-mock";

// Override the invoke function in development
if (process.env.NODE_ENV === "development") {
  require("@tauri-apps/api/core").invoke = async (cmd: string, args?: Record<string, unknown>) => {
    return mockInvoke(cmd, args);
  };
}

export default function ChecklistPage() {
  const [items, setItems] = useState<ChecklistItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    listChecklistItems({}).then((data) => {
      setItems(data);
      setLoading(false);
    });
  }, []);

  const handleDelete = async (id: number) => {
    await deleteChecklistItem(id);
    setItems(items.filter((i) => i.id !== id));
  };

  if (loading) {
    return <div>Loading...</div>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Checklist Items</h1>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          Add Item
        </Button>
      </div>

      <div className="grid gap-4">
        {items.map((item) => (
          <Card key={item.id}>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>{item.name}</CardTitle>
              <div className="flex items-center gap-2">
                <Badge variant={item.status === "active" ? "default" : "secondary"}>
                  {item.status}
                </Badge>
                <Badge variant="outline">{item.target_level}</Badge>
              </div>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground line-clamp-2">{item.prompt}</p>
              <div className="flex justify-end gap-2 mt-4">
                <Button variant="outline" size="sm">
                  <Pencil className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm" onClick={() => handleDelete(item.id)}>
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
