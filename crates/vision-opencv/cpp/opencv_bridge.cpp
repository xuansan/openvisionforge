#include "bridge_common.hpp"

#include "color_ops.cpp"
#include "draw_ops.cpp"
#include "edge_ops.cpp"
#include "enhance_ops.cpp"
#include "filter_ops.cpp"
#include "image_ops.cpp"
#include "measure_ops.cpp"
#include "morph_ops.cpp"
#include "threshold_ops.cpp"
#include "transform_ops.cpp"

static std::string dispatch(const std::vector<std::string>& args) {
    if (args.empty()) {
        fail("empty command");
    }

    const std::string& command = args[0];
    if (command == "metadata") {
        return op_metadata(args);
    }
    if (command == "resize") {
        return op_resize(args);
    }
    if (command == "crop") {
        return op_crop(args);
    }
    if (command == "gray") {
        return op_gray(args);
    }
    if (command == "color_convert") {
        return op_color_convert(args);
    }
    if (command == "threshold") {
        return op_threshold(args);
    }
    if (command == "threshold_otsu") {
        return op_threshold_otsu(args);
    }
    if (command == "threshold_triangle") {
        return op_threshold_triangle(args);
    }
    if (command == "threshold_adaptive") {
        return op_threshold_adaptive(args);
    }
    if (command == "threshold_range") {
        return op_threshold_range(args);
    }
    if (command == "blur") {
        return op_blur(args);
    }
    if (command == "gaussian") {
        return op_gaussian(args);
    }
    if (command == "median") {
        return op_median(args);
    }
    if (command == "sharpen") {
        return op_sharpen(args);
    }
    if (command == "enhance") {
        return op_enhance(args);
    }
    if (command == "draw") {
        return op_draw(args);
    }
    if (command == "morph") {
        return op_morph(args);
    }
    if (command == "ratio") {
        return op_ratio(args);
    }
    if (command == "contours") {
        return op_contours(args);
    }
    if (command == "fit_shape") {
        return op_fit_shape(args);
    }
    if (command == "rotate") {
        return op_rotate(args);
    }
    if (command == "flip") {
        return op_flip(args);
    }
    if (command == "canny") {
        return op_canny(args);
    }
    if (command == "write") {
        return op_write(args);
    }

    fail("unknown command: " + command);
    return "{}";
}

static void run_worker() {
    std::string line;
    while (std::getline(std::cin, line)) {
        try {
            std::cout << dispatch(split_tab(line)) << std::endl;
        } catch (const std::exception& err) {
            std::cout << "{\"error\":\"" << json_escape(err.what()) << "\"}" << std::endl;
        }
    }
}

int main(int argc, char** argv) {
    if (argc >= 2 && std::string(argv[1]) == "worker") {
        run_worker();
        return 0;
    }

    if (argc < 3) {
        fail("usage: opencv_bridge <metadata|gray|threshold|ratio|rotate|write|worker> <args...>");
    }

    std::vector<std::string> args;
    for (int i = 1; i < argc; ++i) {
        args.push_back(argv[i]);
    }
    std::cout << dispatch(args) << std::endl;
    return 0;
}
