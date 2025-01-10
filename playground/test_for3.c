int main(void) {
  int b = 0;
  // No post
  for(int a = 0; a < 10;) {
    a = a+1;
    b = a;
  }

  return b;
}
