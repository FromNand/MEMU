#include "common.h"
#include <gtk-3.0/gtk/gtk.h>
#include <sys/time.h>

// SMBの左端の背景がおかしい
// SMBで変な背景とスプライトが表示される (タイルがスライドしたり、色が変わったりしているため、ネームテーブルが疑われる)

// 左端8ピクセルのスプライトの非表示
// 背景の後ろに隠れるスプライト
// スプライトオーバーフローの実装
// スプライトゼロヒットの判定と背景レンダリングをスキャンライン毎に行う
// 8*16モードの垂直反転はタイルの交換である (8*16モードでは、oam_dataのバイト1を使用してパターンテーブルを探す)

// PPUレジスタの機能を確認し実装する

#define FPS (60)

int draw_count;
GtkWidget *drawing_area;
unsigned char frame[BYTE_PER_PIXEL * SCREEN_PIXEL_WIDTH * SCREEN_PIXEL_HEIGHT];

extern unsigned char button_status;

void init_nes(char *file_name);
gboolean run_nes(gpointer data);

void error(char *message, ...) {
    va_list argument;
    va_start(argument, message);
    vfprintf(stderr, message, argument);
    va_end(argument);
    exit(EXIT_FAILURE);
}

void open_file(GtkWidget *widget, gpointer data) {
    GtkWidget *dialog = gtk_file_chooser_dialog_new("Open File", GTK_WINDOW(data), GTK_FILE_CHOOSER_ACTION_OPEN, \
                                                    "_Open", GTK_RESPONSE_ACCEPT, "_Cancel", GTK_RESPONSE_CANCEL, NULL);
    gtk_file_chooser_set_current_folder(GTK_FILE_CHOOSER(dialog), "./rom");
    if(gtk_dialog_run(GTK_DIALOG(dialog)) == GTK_RESPONSE_ACCEPT) {
        static guint id;
        if(id) g_source_remove(id);
        init_nes(gtk_file_chooser_get_filename(GTK_FILE_CHOOSER(dialog)));
        id = g_idle_add(run_nes, NULL);
    }
    gtk_widget_destroy(dialog);
}

gboolean key_press(GtkWidget *widget, GdkEventKey *event, gpointer data) {
    switch(event->keyval) {
        case GDK_KEY_Escape:
            gtk_main_quit();
            break;
        case GDK_KEY_j:
            button_status |= 0x01;
            break;
        case GDK_KEY_k:
            button_status |= 0x02;
            break;
        case GDK_KEY_space:
            button_status |= 0x04;
            break;
        case GDK_KEY_Return:
            button_status |= 0x08;
            break;
        case GDK_KEY_w:
            button_status |= 0x10;
            break;
        case GDK_KEY_s:
            button_status |= 0x20;
            break;
        case GDK_KEY_a:
            button_status |= 0x40;
            break;
        case GDK_KEY_d:
            button_status |= 0x80;
            break;
    }
    return TRUE;
}

gboolean key_release(GtkWidget *widget, GdkEventKey *event, gpointer data) {
    switch(event->keyval) {
        case GDK_KEY_j:
            button_status &= ~0x01;
            break;
        case GDK_KEY_k:
            button_status &= ~0x02;
            break;
        case GDK_KEY_space:
            button_status &= ~0x04;
            break;
        case GDK_KEY_Return:
            button_status &= ~0x08;
            break;
        case GDK_KEY_w:
            button_status &= ~0x10;
            break;
        case GDK_KEY_s:
            button_status &= ~0x20;
            break;
        case GDK_KEY_a:
            button_status &= ~0x40;
            break;
        case GDK_KEY_d:
            button_status &= ~0x80;
            break;
    }
    return TRUE;
}

gboolean draw(GtkWidget *widget, cairo_t *cairo, gpointer data) {
    draw_count += 1;
    cairo_surface_t *surface = cairo_image_surface_create_for_data(frame, CAIRO_FORMAT_RGB24, SCREEN_PIXEL_WIDTH, SCREEN_PIXEL_HEIGHT, BYTE_PER_PIXEL * SCREEN_PIXEL_WIDTH);
    cairo_set_source_surface(cairo, surface, 0, 0);
    cairo_paint(cairo);
    cairo_surface_destroy(surface);
    struct timeval current_time;
    static struct timeval last_time;
    gettimeofday(&current_time, NULL);
    long sleep_time = (1000 * 1000 / FPS) - (1000 * 1000 * (current_time.tv_sec - last_time.tv_sec) + (current_time.tv_usec - last_time.tv_usec));
    if(sleep_time > 0) {
        g_usleep(sleep_time);
    }
    gettimeofday(&last_time, NULL);
    return TRUE;
}

gboolean show_fps(gpointer data) {
    char s[256];
    sprintf(s, "MEMU [%d]", draw_count);
    gtk_window_set_title(GTK_WINDOW(data), s);
    draw_count = 0;
    return G_SOURCE_CONTINUE;
}

int main(int argc, char **argv) {
    gtk_init(&argc, &argv);

    GtkWidget *menu_bar = gtk_menu_bar_new();
    GtkWidget *open_menu_item = gtk_menu_item_new_with_label("Open");
    GtkWidget *exit_menu_item = gtk_menu_item_new_with_label("Exit");
    gtk_menu_shell_append(GTK_MENU_SHELL(menu_bar), open_menu_item);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu_bar), exit_menu_item);

    drawing_area = gtk_drawing_area_new();
    gtk_widget_set_size_request(drawing_area, SCREEN_PIXEL_WIDTH, SCREEN_PIXEL_HEIGHT);

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_box_pack_start(GTK_BOX(box), menu_bar, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), drawing_area, FALSE, FALSE, 0);

    GtkWidget *window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(window), "MEMU");
    gtk_window_set_position(GTK_WINDOW(window), GTK_WIN_POS_CENTER);
    gtk_window_set_default_size(GTK_WINDOW(window), SCREEN_PIXEL_WIDTH, SCREEN_PIXEL_HEIGHT);
    gtk_container_add(GTK_CONTAINER(window), box);

    g_signal_connect(open_menu_item, "activate", G_CALLBACK(open_file), window);
    g_signal_connect(exit_menu_item, "activate", G_CALLBACK(gtk_main_quit), NULL);
    g_signal_connect(drawing_area, "draw", G_CALLBACK(draw), NULL);
    g_signal_connect(window, "key-press-event", G_CALLBACK(key_press), NULL);
    g_signal_connect(window, "key-release-event", G_CALLBACK(key_release), NULL);
    g_signal_connect(window, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    g_timeout_add(1000, show_fps, window);

    gtk_widget_show_all(window);
    gtk_main();
    return 0;
}
