#include "common.h"
#include <gtk-3.0/gtk/gtk.h>
#include <sys/time.h>

// FCEUXとNintendulatorを参考にする
// デバッグウィンドウを実装する

#define PIXEL_SIZE (3)
#define FPS (60)

int draw_count;
GtkWidget *drawing_area;
unsigned char frame[BYTE_PER_PIXEL * SCREEN_WIDTH * SCREEN_HEIGHT];

void init_nes(char *file_name);
gboolean run_nes(gpointer data);
void render(void);

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

gboolean key_input(GtkWidget *widget, GdkEventKey *event, gpointer data) {
    switch(event->keyval) {
        case GDK_KEY_Escape:
        case GDK_KEY_w:
        case GDK_KEY_a:
        case GDK_KEY_s:
        case GDK_KEY_d:
        default:
            return TRUE;
    }
}

gboolean draw(GtkWidget *widget, cairo_t *cairo, gpointer data) {
    draw_count += 1;
    render();
    cairo_surface_t *surface = cairo_image_surface_create_for_data(frame, CAIRO_FORMAT_RGB24, PIXEL_SIZE * SCREEN_WIDTH, PIXEL_SIZE * SCREEN_HEIGHT, BYTE_PER_PIXEL * PIXEL_SIZE * SCREEN_WIDTH);
    cairo_set_source_surface(cairo, surface, 0, 0);
    for(int py = 0; py < SCREEN_HEIGHT; py++) {
        for(int px = 0; px < SCREEN_WIDTH; px++) {
            int index = BYTE_PER_PIXEL * (px + SCREEN_WIDTH * py);
            cairo_set_source_rgb(cairo, frame[index + 0] / 255.0, frame[index + 1] / 255.0, frame[index + 2] / 255.0);
            cairo_rectangle(cairo, PIXEL_SIZE * px, PIXEL_SIZE * py, PIXEL_SIZE, PIXEL_SIZE);
            cairo_fill(cairo);
        }
    }
    cairo_surface_destroy(surface);

    static struct timeval last_time;
    struct timeval current_time;
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
    gtk_widget_set_size_request(drawing_area, PIXEL_SIZE * SCREEN_WIDTH, PIXEL_SIZE * SCREEN_HEIGHT);

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_box_pack_start(GTK_BOX(box), menu_bar, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), drawing_area, FALSE, FALSE, 0);

    GtkWidget *window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(window), "MEMU");
    gtk_window_set_position(GTK_WINDOW(window), GTK_WIN_POS_CENTER);
    gtk_window_set_default_size(GTK_WINDOW(window), PIXEL_SIZE * SCREEN_WIDTH, PIXEL_SIZE * SCREEN_HEIGHT);
    gtk_container_add(GTK_CONTAINER(window), box);

    g_signal_connect(open_menu_item, "activate", G_CALLBACK(open_file), window);
    g_signal_connect(exit_menu_item, "activate", G_CALLBACK(gtk_main_quit), NULL);
    g_signal_connect(drawing_area, "draw", G_CALLBACK(draw), NULL);
    g_signal_connect(window, "key-press-event", G_CALLBACK(key_input), NULL);
    g_signal_connect(window, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    g_timeout_add(1000, show_fps, window);

    gtk_widget_show_all(window);
    gtk_main();
    return 0;
}
